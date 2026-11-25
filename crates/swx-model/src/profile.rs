//! Sketch regions → kernel profile loops.

use std::fmt;

use swx_kernel::{ProfileLoop, Segment2};
use swx_sketch::profile::{Loop, Region};
use swx_sketch::{GeomId, Geometry, Point, Sketch};

/// A region converted for [`Kernel::make_face`](swx_kernel::Kernel::make_face)
/// and [`Kernel::make_face_tracked`](swx_kernel::Kernel::make_face_tracked).
#[derive(Debug, Clone, PartialEq)]
pub struct RegionProfile {
    /// The outer boundary.
    pub outer: ProfileLoop,
    /// The holes, in the same order as [`Region::holes`].
    pub holes: Vec<ProfileLoop>,
    /// `sources[i][j]` is the sketch entity of segment `j` of loop `i`, where
    /// loop 0 is [`outer`](Self::outer) and loop `i + 1` is `holes[i]`. This
    /// is the same nesting as the edges `make_face_tracked` returns.
    pub sources: Vec<Vec<GeomId>>,
}

/// Why a region could not be converted.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelError {
    /// The region names an entity the sketch does not have.
    UnknownEntity(GeomId),
    /// The region names a point, which can't bound a face.
    NotACurve(GeomId),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::UnknownEntity(id) => {
                write!(
                    f,
                    "region names sketch entity {}, which does not exist",
                    id.0
                )
            }
            ModelError::NotACurve(id) => {
                write!(f, "region names sketch entity {}, which is a point", id.0)
            }
        }
    }
}

impl std::error::Error for ModelError {}

/// Converts `region` of the solved `sketch` into kernel profile loops.
///
/// Segment `j` of each loop is edge `j` of the region's loop, in order, with
/// coordinates read from the sketch ([`Sketch::get`]). A line traversed
/// reversed has its `start` and `end` swapped; an arc keeps its center,
/// radius and angles (the kernel accepts segments running backwards); a
/// circle becomes [`Segment2::Circle`].
///
/// # Errors
///
/// [`ModelError::UnknownEntity`] when an edge names an entity the sketch
/// lacks, [`ModelError::NotACurve`] when it names a point.
pub fn region_to_profile(sketch: &Sketch, region: &Region) -> Result<RegionProfile, ModelError> {
    let (outer, outer_src) = convert_loop(sketch, &region.outer)?;
    let mut holes = Vec::with_capacity(region.holes.len());
    let mut sources = vec![outer_src];
    for hole in &region.holes {
        let (l, src) = convert_loop(sketch, hole)?;
        holes.push(l);
        sources.push(src);
    }
    Ok(RegionProfile {
        outer,
        holes,
        sources,
    })
}

fn xy(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

fn convert_loop(sketch: &Sketch, lp: &Loop) -> Result<(ProfileLoop, Vec<GeomId>), ModelError> {
    let mut segments = Vec::with_capacity(lp.edges.len());
    let mut sources = Vec::with_capacity(lp.edges.len());
    for edge in &lp.edges {
        let geom = sketch
            .get(edge.geom)
            .map_err(|_| ModelError::UnknownEntity(edge.geom))?;
        let seg = match geom {
            Geometry::Point(_) => return Err(ModelError::NotACurve(edge.geom)),
            Geometry::Line(l) if edge.reversed => Segment2::Line {
                start: xy(l.end),
                end: xy(l.start),
            },
            Geometry::Line(l) => Segment2::Line {
                start: xy(l.start),
                end: xy(l.end),
            },
            Geometry::Arc(a) => Segment2::Arc {
                center: xy(a.center),
                radius: a.radius,
                start_angle: a.start_angle,
                end_angle: a.end_angle,
            },
            Geometry::Circle(c) => Segment2::Circle {
                center: xy(c.center),
                radius: c.radius,
            },
        };
        segments.push(seg);
        sources.push(edge.geom);
    }
    Ok((ProfileLoop { segments }, sources))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};
    use swx_kernel::LOOP_GAP_TOLERANCE;
    use swx_sketch::profile::{profiles, OrientedEdge, ProfileOptions};
    use swx_sketch::{Arc, Circle, Line};

    const TOL: f64 = 1e-9;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }

    fn line(s: &mut Sketch, a: (f64, f64), b: (f64, f64)) -> GeomId {
        s.add(Geometry::Line(Line::new(p(a.0, a.1), p(b.0, b.1)).unwrap()))
            .unwrap()
    }

    fn ends(seg: &Segment2) -> Option<([f64; 2], [f64; 2])> {
        match *seg {
            Segment2::Line { start, end } => Some((start, end)),
            Segment2::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            } => Some((
                [
                    center[0] + radius * start_angle.cos(),
                    center[1] + radius * start_angle.sin(),
                ],
                [
                    center[0] + radius * end_angle.cos(),
                    center[1] + radius * end_angle.sin(),
                ],
            )),
            Segment2::Circle { .. } => None,
        }
    }

    /// `½ ∮ (x dy − y dx)` of one segment run forwards (Green's theorem, as
    /// in `swx_sketch::profile`).
    fn area_term(seg: &Segment2) -> f64 {
        match *seg {
            Segment2::Line { start, end } => 0.5 * (start[0] * end[1] - start[1] * end[0]),
            Segment2::Arc {
                center: c,
                radius: r,
                start_angle: a0,
                end_angle: a1,
            } => {
                0.5 * (r * r * (a1 - a0) + r * c[0] * (a1.sin() - a0.sin())
                    - r * c[1] * (a1.cos() - a0.cos()))
            }
            Segment2::Circle { radius, .. } => PI * radius * radius,
        }
    }

    fn near(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).hypot(a[1] - b[1]) <= LOOP_GAP_TOLERANCE
    }

    /// Walks the loop, letting each segment run either way, and returns its
    /// signed area, or `None` when consecutive segments don't meet.
    fn chained_area(l: &ProfileLoop) -> Option<f64> {
        let segs = &l.segments;
        if let [seg @ Segment2::Circle { .. }] = segs.as_slice() {
            return Some(area_term(seg));
        }
        let (s0, e0) = ends(&segs[0])?;
        let (s1, e1) = ends(&segs[1])?;
        let first_forward = near(e0, s1) || near(e0, e1);
        let mut at = if first_forward { e0 } else { s0 };
        let start = if first_forward { s0 } else { e0 };
        let mut area = if first_forward { 1.0 } else { -1.0 } * area_term(&segs[0]);
        for seg in &segs[1..] {
            let (s, e) = ends(seg)?;
            if near(at, s) {
                area += area_term(seg);
                at = e;
            } else if near(at, e) {
                area -= area_term(seg);
                at = s;
            } else {
                return None;
            }
        }
        near(at, start).then_some(area)
    }

    /// Test 1's sketch: rectangle e1–e4 (40 × 25) and a hole circle e5.
    fn plate() -> (Sketch, [GeomId; 5]) {
        let mut s = Sketch::new();
        let e1 = line(&mut s, (0.0, 0.0), (40.0, 0.0));
        let e2 = line(&mut s, (40.0, 0.0), (40.0, 25.0));
        let e3 = line(&mut s, (40.0, 25.0), (0.0, 25.0));
        let e4 = line(&mut s, (0.0, 25.0), (0.0, 0.0));
        let e5 = s
            .add(Geometry::Circle(Circle::new(p(20.0, 12.5), 5.0).unwrap()))
            .unwrap();
        (s, [e1, e2, e3, e4, e5])
    }

    fn only_region(s: &Sketch) -> Region {
        let mut regions = profiles(s, &ProfileOptions::default()).unwrap();
        assert_eq!(regions.len(), 1);
        regions.remove(0)
    }

    #[test]
    fn rectangle_with_hole_converts_with_sources() {
        let (s, [e1, e2, e3, e4, e5]) = plate();
        let region = only_region(&s);
        let prof = region_to_profile(&s, &region).unwrap();

        assert_eq!(prof.outer.segments.len(), 4);
        assert!(prof
            .outer
            .segments
            .iter()
            .all(|seg| matches!(seg, Segment2::Line { .. })));
        let outer_area = chained_area(&prof.outer).expect("outer loop chains");

        assert_eq!(prof.holes.len(), 1);
        match prof.holes[0].segments.as_slice() {
            [Segment2::Circle { center, radius }] => {
                assert!((center[0] - 20.0).abs() <= TOL && (center[1] - 12.5).abs() <= TOL);
                assert!((radius - 5.0).abs() <= TOL);
            }
            other => panic!("hole should be one circle, got {other:?}"),
        }
        let hole_area = chained_area(&prof.holes[0]).unwrap();
        let area = outer_area.abs() - hole_area.abs();
        assert!((area - (1000.0 - 25.0 * PI)).abs() <= TOL, "area {area}");

        assert_eq!(prof.sources.len(), 2);
        let outer_src: Vec<GeomId> = region.outer.edges.iter().map(|e| e.geom).collect();
        assert_eq!(prof.sources[0], outer_src);
        let mut sorted = prof.sources[0].clone();
        sorted.sort_by_key(|g| g.0);
        assert_eq!(sorted, vec![e1, e2, e3, e4]);
        assert_eq!(prof.sources[1], vec![e5]);
    }

    #[test]
    fn slot_with_reversed_arc_chains_and_keeps_area() {
        // A slot 35 long, 10 wide: a convex cap on the right and a concave
        // cap on the left, so the left arc runs clockwise in the CCW outline.
        // The bottom line is drawn right to left, so it runs reversed too.
        let mut s = Sketch::new();
        line(&mut s, (30.0, 0.0), (-5.0, 0.0));
        s.add(Geometry::Arc(
            Arc::new(p(30.0, 5.0), 5.0, -FRAC_PI_2, FRAC_PI_2).unwrap(),
        ))
        .unwrap();
        line(&mut s, (30.0, 10.0), (-5.0, 10.0));
        s.add(Geometry::Arc(
            Arc::new(p(-5.0, 5.0), 5.0, -FRAC_PI_2, FRAC_PI_2).unwrap(),
        ))
        .unwrap();
        let region = only_region(&s);
        assert!(
            region.outer.edges.iter().any(|e| e.reversed),
            "test needs a reversed edge"
        );
        let prof = region_to_profile(&s, &region).unwrap();
        assert_eq!(prof.outer.segments.len(), 4);
        let area = chained_area(&prof.outer).expect("slot loop chains");
        assert!(
            (area - region.area()).abs() <= TOL,
            "{area} vs {}",
            region.area()
        );
        assert!((area - 350.0).abs() <= TOL, "area {area}");
    }

    #[test]
    fn corner_cut_adds_segment_and_keeps_hole_source() {
        // M3-04 spec, problem case 2: a new line e6 cuts the corner at (40, 0).
        let mut s = Sketch::new();
        let e1 = line(&mut s, (0.0, 0.0), (35.0, 0.0));
        let e2 = line(&mut s, (40.0, 5.0), (40.0, 25.0));
        line(&mut s, (40.0, 25.0), (0.0, 25.0));
        line(&mut s, (0.0, 25.0), (0.0, 0.0));
        let e5 = s
            .add(Geometry::Circle(Circle::new(p(20.0, 12.5), 5.0).unwrap()))
            .unwrap();
        let e6 = line(&mut s, (35.0, 0.0), (40.0, 5.0));
        let region = only_region(&s);
        let prof = region_to_profile(&s, &region).unwrap();

        assert_eq!(prof.outer.segments.len(), 5);
        assert!(prof.sources[0].contains(&e6));
        assert!(prof.sources[0].contains(&e1) && prof.sources[0].contains(&e2));
        assert_eq!(prof.sources[1], vec![e5]);
        let area = chained_area(&prof.outer).unwrap().abs();
        assert!((area - (1000.0 - 12.5)).abs() <= TOL, "area {area}");
    }

    #[test]
    fn point_or_unknown_entity_is_an_error() {
        let (mut s, [e1, e2, e3, ..]) = plate();
        let pt = s.add(Geometry::Point(p(1.0, 1.0))).unwrap();
        let edge = |geom| OrientedEdge {
            geom,
            reversed: false,
        };
        let region_with = |bad| Region {
            outer: Loop {
                edges: vec![edge(e1), edge(e2), edge(bad), edge(e3)],
                signed_area: 1.0,
            },
            holes: vec![],
        };

        let err = region_to_profile(&s, &region_with(pt)).unwrap_err();
        assert_eq!(err, ModelError::NotACurve(pt));
        assert!(err.to_string().contains("point"));

        let missing = GeomId(99);
        let err = region_to_profile(&s, &region_with(missing)).unwrap_err();
        assert_eq!(err, ModelError::UnknownEntity(missing));
        let _: &dyn std::error::Error = &err;

        // A bad hole is caught too.
        let region = Region {
            outer: only_region(&plate().0).outer,
            holes: vec![Loop {
                edges: vec![edge(pt)],
                signed_area: -1.0,
            }],
        };
        assert_eq!(
            region_to_profile(&s, &region).unwrap_err(),
            ModelError::NotACurve(pt)
        );
    }

    #[cfg(feature = "occt")]
    #[test]
    fn plate_face_area_and_tracked_edges_match_sources() {
        use swx_kernel::{Kernel, OcctKernel, Workplane};

        let (s, _) = plate();
        let region = only_region(&s);
        let prof = region_to_profile(&s, &region).unwrap();
        let k = OcctKernel;
        let (face, edges) = k
            .make_face_tracked(&Workplane::XY, &prof.outer, &prof.holes)
            .unwrap();
        let area = k.surface_area(&face).unwrap();
        assert!((area - (1000.0 - 25.0 * PI)).abs() <= 1e-6, "area {area}");
        let shape: Vec<usize> = edges.iter().map(Vec::len).collect();
        let want: Vec<usize> = prof.sources.iter().map(Vec::len).collect();
        assert_eq!(shape, want);
    }
}
