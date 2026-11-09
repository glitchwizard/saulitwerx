//! 2D profiles on a workplane, and extrude extents.
//!
//! [`Workplane`], [`Segment2`], [`ProfileLoop`] and [`Extent`] are the inputs
//! of [`Kernel::make_face`](crate::Kernel::make_face) and
//! [`Kernel::extrude`](crate::Kernel::extrude). Everything here is pure Rust:
//! inputs are validated and loops are put in a canonical order before any
//! backend call, so the error cases are testable without OCCT.

use std::f64::consts::TAU;

use crate::{KernelError, Point3, SubShape};

/// Gap allowed between consecutive segments of a loop.
pub const LOOP_GAP_TOLERANCE: f64 = 1e-7;

/// Largest `|x_dir · normal|` (after normalizing both) that still counts as perpendicular.
pub const PERPENDICULAR_TOLERANCE: f64 = 1e-9;

/// A plane in model space with a 2D coordinate system.
///
/// A 2D point `(u, v)` maps to `origin + u * x_dir + v * (normal × x_dir)`.
/// Counterclockwise is as seen looking down `normal` onto the plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Workplane {
    /// Model-space position of the 2D origin.
    pub origin: Point3,
    /// Direction of the 2D +u axis. Normalized before use.
    pub x_dir: [f64; 3],
    /// Plane normal; positive extrude distances go this way. Normalized before use.
    pub normal: [f64; 3],
}

impl Workplane {
    /// The XY plane: u along +X, v along +Y, normal +Z.
    pub const XY: Workplane = Workplane {
        origin: [0.0; 3],
        x_dir: [1.0, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    };
    /// The XZ plane: u along +X, v along +Z, normal −Y.
    pub const XZ: Workplane = Workplane {
        origin: [0.0; 3],
        x_dir: [1.0, 0.0, 0.0],
        normal: [0.0, -1.0, 0.0],
    };
    /// The YZ plane: u along +Y, v along +Z, normal +X.
    pub const YZ: Workplane = Workplane {
        origin: [0.0; 3],
        x_dir: [0.0, 1.0, 0.0],
        normal: [1.0, 0.0, 0.0],
    };
}

/// One piece of a profile loop, in workplane coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment2 {
    /// Straight line from `start` to `end`.
    Line {
        /// First endpoint.
        start: [f64; 2],
        /// Second endpoint.
        end: [f64; 2],
    },
    /// Circular arc running counterclockwise from `start_angle` to
    /// `end_angle` (radians, measured from the workplane's +u axis).
    Arc {
        /// Center.
        center: [f64; 2],
        /// Radius, > 0.
        radius: f64,
        /// Angle of the first endpoint.
        start_angle: f64,
        /// Angle of the second endpoint.
        end_angle: f64,
    },
    /// Full circle. Must be the only segment of its loop.
    Circle {
        /// Center.
        center: [f64; 2],
        /// Radius, > 0.
        radius: f64,
    },
}

/// A closed chain of segments. Each segment's end must meet the next one's
/// start (either segment may run backwards), and the last must meet the first.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProfileLoop {
    /// The segments in chain order.
    pub segments: Vec<Segment2>,
}

/// How far an extrude goes from the workplane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extent {
    /// Signed distance along the workplane normal; negative goes the other way.
    Distance(f64),
    /// Total length, split equally on both sides of the workplane.
    Symmetric(f64),
    /// Through the whole target body, in the given direction. Only
    /// [`extrude_feature`](crate::extrude_feature) with
    /// [`ExtrudeMode::Cut`](crate::ExtrudeMode::Cut) accepts it, because the
    /// length is measured from the target; [`Kernel::extrude`](crate::Kernel::extrude)
    /// rejects it.
    ThroughAll(ThroughDirection),
    /// From signed distance `start` to signed distance `end` along the
    /// workplane normal, in either order. Both must be finite and differ.
    FromTo(f64, f64),
    /// From the workplane to the plane of a planar face of the target body,
    /// which must be parallel to the workplane. The face may be on either
    /// side. Like [`Extent::ThroughAll`], it needs a target:
    /// [`extrude_feature`](crate::extrude_feature) resolves it (any mode but
    /// [`ExtrudeMode::NewSolid`](crate::ExtrudeMode::NewSolid)), and
    /// [`Kernel::extrude`](crate::Kernel::extrude) rejects it.
    ToFace(SubShape),
    /// From the plane of the first face to the plane of the second, both
    /// planar faces of the target body parallel to the workplane; the
    /// workplane's own position doesn't matter. Resolved and rejected like
    /// [`Extent::ToFace`].
    BetweenFaces(SubShape, SubShape),
}

/// Which way a [`Extent::ThroughAll`] extrude goes from the workplane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThroughDirection {
    /// Along the workplane normal.
    Normal,
    /// Against the workplane normal.
    Reversed,
    /// Both ways.
    Both,
}

impl Extent {
    /// Returns `(offset, length)`: the prism starts `offset` along the normal
    /// and runs `length > 0` along it.
    ///
    /// [`Extent::ThroughAll`] is [`KernelError::InvalidInput`]: it has no
    /// length until [`extrude_feature`](crate::extrude_feature) resolves it
    /// against a target.
    #[cfg_attr(not(feature = "occt"), allow(dead_code))]
    pub(crate) fn offset_and_length(self) -> Result<(f64, f64), KernelError> {
        let check = |name: &str, v: f64| {
            if v.is_finite() && v != 0.0 {
                Ok(v)
            } else {
                Err(KernelError::InvalidInput(format!(
                    "{name} must be finite and non-zero, got {v}"
                )))
            }
        };
        match self {
            Extent::Distance(d) => {
                let d = check("extrude distance", d)?;
                Ok(if d < 0.0 { (d, -d) } else { (0.0, d) })
            }
            Extent::Symmetric(l) => {
                let l = check("symmetric extrude length", l)?.abs();
                Ok((-l / 2.0, l))
            }
            Extent::FromTo(start, end) => {
                if !start.is_finite() || !end.is_finite() {
                    return Err(KernelError::InvalidInput(format!(
                        "extrude start and end must be finite, got {start} and {end}"
                    )));
                }
                check("extrude length (end − start)", end - start)?;
                Ok((start.min(end), (end - start).abs()))
            }
            Extent::ThroughAll(_) => Err(KernelError::InvalidInput(
                "a through-all extent needs a target body to measure; \
                 use extrude_feature with the cut mode"
                    .into(),
            )),
            Extent::ToFace(_) | Extent::BetweenFaces(..) => Err(KernelError::InvalidInput(
                "a to-face or between-faces extent needs a target body to measure; \
                 use extrude_feature with the join, cut or intersect mode"
                    .into(),
            )),
        }
    }
}

/// A revolve axis: the line through `origin` along `dir`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis {
    /// A point on the axis.
    pub origin: Point3,
    /// Axis direction. Normalized before use; a positive revolve angle turns
    /// counterclockwise looking down it (right-hand rule).
    pub dir: [f64; 3],
}

/// Largest amount a revolve angle may exceed 2π and still count as a full turn.
const FULL_TURN_TOLERANCE: f64 = 1e-12;

/// Validates revolve inputs and returns the axis origin, the normalized axis
/// direction and the angle (clamped to 2π when within
/// [`FULL_TURN_TOLERANCE`] above it).
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn prepare_revolve(
    axis: Axis,
    angle: f64,
) -> Result<(Point3, [f64; 3], f64), KernelError> {
    finite("revolve axis origin", &axis.origin)?;
    let dir = normalized("revolve axis direction", axis.dir)?;
    if !angle.is_finite() || angle <= 0.0 || angle > TAU + FULL_TURN_TOLERANCE {
        return Err(invalid(format!(
            "revolve angle must be in (0, 2π] radians, got {angle}"
        )));
    }
    Ok((axis.origin, dir, angle.min(TAU)))
}

/// Absolute tolerance, in model units, of the revolve axis and profile checks
/// (the same value as OCCT's `Precision::Confusion`).
pub const REVOLVE_AXIS_TOLERANCE: f64 = 1e-7;

/// What the backend reports about a planar profile face for the revolve
/// checks (see [`check_revolve_profile`]).
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RevolveProfile {
    /// A point on the face's plane.
    pub plane_point: Point3,
    /// The plane's unit normal `n`.
    pub normal: [f64; 3],
    /// Smallest `(P − origin) · (dir × n)` over the face's points `P`.
    pub side_min: f64,
    /// Largest `(P − origin) · (dir × n)` over the face's points `P`.
    pub side_max: f64,
}

/// Rejects a revolve whose axis is not in the profile's plane, or whose
/// profile has points strictly on both sides of the axis.
///
/// `origin` and the unit `dir` come from [`prepare_revolve`]. The axis is in
/// the plane when `dir` is perpendicular to the plane's unit normal `n`
/// (`|dir · n| ≤ tol`) and `origin` is on the plane
/// (`|(origin − plane_point) · n| ≤ tol`). Then `w = dir × n` is the unit
/// vector in the plane perpendicular to the axis, and the sign of
/// `(P − origin) · w` says which side of the axis a profile point `P` is on.
/// The profile crosses the axis when that value is below `−tol` somewhere and
/// above `tol` somewhere else. A profile that only touches the axis is valid.
/// `tol` is [`REVOLVE_AXIS_TOLERANCE`].
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn check_revolve_profile(
    origin: Point3,
    dir: [f64; 3],
    profile: &RevolveProfile,
) -> Result<(), KernelError> {
    let tol = REVOLVE_AXIS_TOLERANCE;
    let n = profile.normal;
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let offset = [
        origin[0] - profile.plane_point[0],
        origin[1] - profile.plane_point[1],
        origin[2] - profile.plane_point[2],
    ];
    if dot(dir, n).abs() > tol || dot(offset, n).abs() > tol {
        return Err(invalid(format!(
            "revolve axis (origin {origin:?}, direction {dir:?}) is not in the profile's plane \
             (through {:?}, normal {n:?})",
            profile.plane_point
        )));
    }
    if profile.side_min < -tol && profile.side_max > tol {
        return Err(invalid(format!(
            "the profile crosses the revolve axis (origin {origin:?}, direction {dir:?}): \
             it reaches {} on one side and {} on the other",
            -profile.side_min, profile.side_max
        )));
    }
    Ok(())
}

/// Segment kinds in the encoding the OCCT shim reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeKind {
    Line = 0,
    ArcCcw = 1,
    ArcCw = 2,
    CircleCcw = 3,
    CircleCw = 4,
}

/// A segment as traversed in its loop, with snapped shared endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Edge2 {
    pub kind: EdgeKind,
    /// Where the traversal enters the segment (unused for circles).
    pub from: [f64; 2],
    /// Where the traversal leaves the segment (unused for circles).
    pub to: [f64; 2],
    /// Arc or circle center (unused for lines).
    pub center: [f64; 2],
    /// Arc or circle radius (unused for lines).
    pub radius: f64,
}

impl Edge2 {
    /// Signed sweep in radians: positive counterclockwise.
    fn sweep(&self) -> f64 {
        let angle = |p: [f64; 2]| (p[1] - self.center[1]).atan2(p[0] - self.center[0]);
        let ccw = (angle(self.to) - angle(self.from)).rem_euclid(TAU);
        match self.kind {
            EdgeKind::Line => 0.0,
            EdgeKind::ArcCcw => ccw,
            EdgeKind::ArcCw => -(TAU - ccw).rem_euclid(TAU),
            EdgeKind::CircleCcw => TAU,
            EdgeKind::CircleCw => -TAU,
        }
    }

    /// This segment's term of the signed area `½∮(x dy − y dx)` (Green's
    /// theorem; e.g. Stewart, *Calculus*, §16.4). For a circular arc with
    /// center `c`, radius `r` and signed sweep `Δ` it is
    /// `½(cx·(y1 − y0) − cy·(x1 − x0) + r²Δ)`.
    fn area_term(&self) -> f64 {
        let [x0, y0] = self.from;
        let [x1, y1] = self.to;
        match self.kind {
            EdgeKind::Line => 0.5 * (x0 * y1 - x1 * y0),
            _ => {
                let [cx, cy] = self.center;
                0.5 * (cx * (y1 - y0) - cy * (x1 - x0) + self.radius * self.radius * self.sweep())
            }
        }
    }

    /// The same segment traversed the other way.
    fn reversed(self) -> Edge2 {
        let kind = match self.kind {
            EdgeKind::Line => EdgeKind::Line,
            EdgeKind::ArcCcw => EdgeKind::ArcCw,
            EdgeKind::ArcCw => EdgeKind::ArcCcw,
            EdgeKind::CircleCcw => EdgeKind::CircleCw,
            EdgeKind::CircleCw => EdgeKind::CircleCcw,
        };
        Edge2 {
            kind,
            from: self.to,
            to: self.from,
            ..self
        }
    }
}

/// A validated face description: normalized plane axes and oriented loops
/// (outer counterclockwise about the normal, holes clockwise).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PreparedFace {
    pub origin: Point3,
    pub x_dir: [f64; 3],
    pub normal: [f64; 3],
    /// The outer loop first, then the holes.
    pub loops: Vec<Vec<Edge2>>,
    /// Per loop: `true` when its edges are in the reverse of the caller's
    /// segment order (edge `k` came from segment `n - 1 - k`).
    pub reversed: Vec<bool>,
}

fn invalid(msg: String) -> KernelError {
    KernelError::InvalidInput(msg)
}

fn finite(name: &str, values: &[f64]) -> Result<(), KernelError> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(invalid(format!("{name} must be finite, got {values:?}")))
    }
}

fn normalized(name: &str, v: [f64; 3]) -> Result<[f64; 3], KernelError> {
    finite(name, &v)?;
    let len = v.iter().map(|c| c * c).sum::<f64>().sqrt();
    if len <= f64::MIN_POSITIVE {
        return Err(invalid(format!("{name} must be non-zero")));
    }
    Ok(v.map(|c| c / len))
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Checks one segment and returns it as a forward [`Edge2`].
fn edge(seg: &Segment2) -> Result<Edge2, KernelError> {
    let check_radius = |r: f64| {
        if r > 0.0 {
            Ok(())
        } else {
            Err(invalid(format!("radius must be > 0, got {r}")))
        }
    };
    match *seg {
        Segment2::Line { start, end } => {
            finite("line", &[start[0], start[1], end[0], end[1]])?;
            if dist(start, end) <= LOOP_GAP_TOLERANCE {
                return Err(invalid(format!(
                    "line from {start:?} to {end:?} has no length"
                )));
            }
            Ok(Edge2 {
                kind: EdgeKind::Line,
                from: start,
                to: end,
                center: [0.0; 2],
                radius: 0.0,
            })
        }
        Segment2::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => {
            finite(
                "arc",
                &[center[0], center[1], radius, start_angle, end_angle],
            )?;
            check_radius(radius)?;
            let at = |a: f64| [center[0] + radius * a.cos(), center[1] + radius * a.sin()];
            let e = Edge2 {
                kind: EdgeKind::ArcCcw,
                from: at(start_angle),
                to: at(end_angle),
                center,
                radius,
            };
            if dist(e.from, e.to) <= LOOP_GAP_TOLERANCE {
                return Err(invalid(format!(
                    "arc from {start_angle} to {end_angle} rad has no sweep; use a circle"
                )));
            }
            Ok(e)
        }
        Segment2::Circle { center, radius } => {
            finite("circle", &[center[0], center[1], radius])?;
            check_radius(radius)?;
            let p = [center[0] + radius, center[1]];
            Ok(Edge2 {
                kind: EdgeKind::CircleCcw,
                from: p,
                to: p,
                center,
                radius,
            })
        }
    }
}

/// Orders `edges` into a closed chain, starting with `edges[0]` traversed
/// forwards or backwards. Returns `None` if they don't chain.
fn chain(edges: &[Edge2], first_reversed: bool) -> Option<Vec<Edge2>> {
    let first = if first_reversed {
        edges[0].reversed()
    } else {
        edges[0]
    };
    let mut out = vec![first];
    for e in &edges[1..] {
        let at = out.last()?.to;
        if dist(e.from, at) <= LOOP_GAP_TOLERANCE {
            out.push(*e);
        } else if dist(e.to, at) <= LOOP_GAP_TOLERANCE {
            out.push(e.reversed());
        } else {
            return None;
        }
    }
    (dist(out.last()?.to, first.from) <= LOOP_GAP_TOLERANCE).then_some(out)
}

/// Validates a loop and returns its edges in traversal order, with each
/// shared endpoint snapped to the midpoint of the two it joins, oriented
/// counterclockwise if `ccw`, else clockwise. The flag is `true` when the
/// edges are in the reverse of the segment order.
fn prepare_loop(
    name: &str,
    lp: &ProfileLoop,
    ccw: bool,
) -> Result<(Vec<Edge2>, bool), KernelError> {
    let edges = lp
        .segments
        .iter()
        .map(edge)
        .collect::<Result<Vec<_>, _>>()?;
    let mut edges = match edges.as_slice() {
        [] => return Err(invalid(format!("{name} loop is empty"))),
        [e] if e.kind == EdgeKind::CircleCcw => edges,
        [_] => {
            return Err(invalid(format!(
                "{name} loop of one segment must be a circle"
            )))
        }
        _ if edges.iter().any(|e| e.kind == EdgeKind::CircleCcw) => {
            return Err(invalid(format!(
                "{name} loop: a circle must be the only segment of its loop"
            )))
        }
        _ => chain(&edges, false)
            .or_else(|| chain(&edges, true))
            .ok_or_else(|| {
                invalid(format!(
                    "{name} loop: consecutive segments don't meet within {LOOP_GAP_TOLERANCE}"
                ))
            })?,
    };
    if edges[0].kind != EdgeKind::CircleCcw {
        let n = edges.len();
        for i in 0..n {
            let j = (i + 1) % n;
            let a = edges[i].to;
            let b = edges[j].from;
            let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
            edges[i].to = mid;
            edges[j].from = mid;
        }
    }
    let area: f64 = edges.iter().map(Edge2::area_term).sum();
    if area.abs() <= LOOP_GAP_TOLERANCE * LOOP_GAP_TOLERANCE {
        return Err(invalid(format!("{name} loop encloses no area")));
    }
    let reverse = (area > 0.0) != ccw;
    if reverse {
        edges = edges.into_iter().rev().map(Edge2::reversed).collect();
    }
    Ok((edges, reverse))
}

/// Validates [`Kernel::make_face`](crate::Kernel::make_face) inputs.
///
/// Checks that are geometric rather than local (a self-intersecting loop, a
/// hole outside the outer loop) are left to the backend.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn prepare_face(
    plane: &Workplane,
    outer: &ProfileLoop,
    holes: &[ProfileLoop],
) -> Result<PreparedFace, KernelError> {
    let mut prepared = prepare_face_plane(plane)?;
    let (mut loops, mut reversed) = (Vec::new(), Vec::new());
    for (i, (name, lp)) in std::iter::once(("outer".to_string(), outer))
        .chain(
            holes
                .iter()
                .enumerate()
                .map(|(i, h)| (format!("hole {i}"), h)),
        )
        .enumerate()
    {
        let (edges, rev) = prepare_loop(&name, lp, i == 0)?;
        loops.push(edges);
        reversed.push(rev);
    }
    prepared.loops = loops;
    prepared.reversed = reversed;
    Ok(prepared)
}

impl PreparedFace {
    /// `[origin, x_dir, normal]` as 9 doubles.
    #[cfg_attr(not(feature = "occt"), allow(dead_code))]
    pub(crate) fn plane_array(&self) -> [f64; 9] {
        let mut out = [0.0; 9];
        out[..3].copy_from_slice(&self.origin);
        out[3..6].copy_from_slice(&self.x_dir);
        out[6..].copy_from_slice(&self.normal);
        out
    }

    /// Edge count of each loop, and every edge as 8 doubles:
    /// `kind, from.u, from.v, to.u, to.v, center.u, center.v, radius`.
    #[cfg_attr(not(feature = "occt"), allow(dead_code))]
    pub(crate) fn encode(&self) -> (Vec<usize>, Vec<f64>) {
        let counts = self.loops.iter().map(Vec::len).collect();
        let edges = self
            .loops
            .iter()
            .flatten()
            .flat_map(|e| {
                [
                    e.kind as i32 as f64,
                    e.from[0],
                    e.from[1],
                    e.to[0],
                    e.to[1],
                    e.center[0],
                    e.center[1],
                    e.radius,
                ]
            })
            .collect();
        (counts, edges)
    }

    /// Model-space point of the 2D workplane point `p`.
    #[cfg_attr(not(feature = "occt"), allow(dead_code))]
    pub(crate) fn point_3d(&self, p: [f64; 2]) -> Point3 {
        let [nx, ny, nz] = self.normal;
        let [xx, xy, xz] = self.x_dir;
        let y_dir = [ny * xz - nz * xy, nz * xx - nx * xz, nx * xy - ny * xx];
        [0, 1, 2].map(|k| self.origin[k] + p[0] * self.x_dir[k] + p[1] * y_dir[k])
    }
}

/// Validates [`Kernel::sweep`](crate::Kernel::sweep) path inputs and returns
/// the path as one open chain (`loops[0]`) in the caller's order, starting at
/// the first segment's start, with each shared endpoint snapped to the
/// midpoint of the two it joins. Later segments may run either way.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn prepare_path(
    plane: &Workplane,
    path: &[Segment2],
) -> Result<PreparedFace, KernelError> {
    let mut prepared = prepare_face_plane(plane)?;
    if path.is_empty() {
        return Err(invalid("sweep path is empty".to_string()));
    }
    if path.iter().any(|s| matches!(s, Segment2::Circle { .. })) {
        return Err(invalid(
            "sweep path: circles (closed paths) are not supported".to_string(),
        ));
    }
    let edges = path.iter().map(edge).collect::<Result<Vec<_>, _>>()?;
    let mut out = vec![edges[0]];
    for (i, e) in edges.iter().enumerate().skip(1) {
        let at = out[out.len() - 1].to;
        let next = if dist(e.from, at) <= LOOP_GAP_TOLERANCE {
            *e
        } else if dist(e.to, at) <= LOOP_GAP_TOLERANCE {
            e.reversed()
        } else {
            return Err(invalid(format!(
                "sweep path: segment {i} doesn't meet segment {} within {LOOP_GAP_TOLERANCE}",
                i - 1
            )));
        };
        let mid = [(at[0] + next.from[0]) / 2.0, (at[1] + next.from[1]) / 2.0];
        let last = out.len() - 1;
        out[last].to = mid;
        out.push(Edge2 { from: mid, ..next });
    }
    prepared.loops = vec![out];
    prepared.reversed = vec![false];
    Ok(prepared)
}

/// Checks `plane` as [`prepare_face`] does and returns it with no loops.
fn prepare_face_plane(plane: &Workplane) -> Result<PreparedFace, KernelError> {
    finite("workplane origin", &plane.origin)?;
    let x_dir = normalized("workplane x_dir", plane.x_dir)?;
    let normal = normalized("workplane normal", plane.normal)?;
    let dot: f64 = x_dir.iter().zip(normal).map(|(a, b)| a * b).sum();
    if dot.abs() > PERPENDICULAR_TOLERANCE {
        return Err(invalid(format!(
            "workplane x_dir and normal must be perpendicular (dot {dot})"
        )));
    }
    Ok(PreparedFace {
        origin: plane.origin,
        x_dir,
        normal,
        loops: Vec::new(),
        reversed: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn line(start: [f64; 2], end: [f64; 2]) -> Segment2 {
        Segment2::Line { start, end }
    }

    fn rect(x0: f64, y0: f64, w: f64, h: f64) -> ProfileLoop {
        let p = [[x0, y0], [x0 + w, y0], [x0 + w, y0 + h], [x0, y0 + h]];
        ProfileLoop {
            segments: (0..4).map(|i| line(p[i], p[(i + 1) % 4])).collect(),
        }
    }

    fn circle(center: [f64; 2], radius: f64) -> ProfileLoop {
        ProfileLoop {
            segments: vec![Segment2::Circle { center, radius }],
        }
    }

    fn area(edges: &[Edge2]) -> f64 {
        edges.iter().map(Edge2::area_term).sum()
    }

    fn is_invalid(r: Result<PreparedFace, KernelError>) -> bool {
        matches!(r, Err(KernelError::InvalidInput(_)))
    }

    #[test]
    fn plate_loops_are_oriented_outer_ccw_hole_cw() {
        let f = prepare_face(
            &Workplane::XY,
            &rect(0.0, 0.0, 40.0, 25.0),
            &[circle([20.0, 12.5], 5.0)],
        )
        .unwrap();
        assert!((area(&f.loops[0]) - 1000.0).abs() < 1e-9);
        assert!((area(&f.loops[1]) + 25.0 * PI).abs() < 1e-9);
        assert_eq!(f.loops[1][0].kind, EdgeKind::CircleCw);
    }

    #[test]
    fn clockwise_outer_is_reversed() {
        let mut lp = rect(0.0, 0.0, 2.0, 3.0);
        lp.segments.reverse();
        for s in &mut lp.segments {
            if let Segment2::Line { start, end } = s {
                std::mem::swap(start, end);
            }
        }
        let f = prepare_face(&Workplane::XY, &lp, &[]).unwrap();
        assert!((area(&f.loops[0]) - 6.0).abs() < 1e-12);
    }

    #[test]
    fn slot_area_and_backwards_arc() {
        // Sides 30 long, semicircular ends of radius 5; the left arc is
        // listed so that the chain must traverse it clockwise.
        let lp = ProfileLoop {
            segments: vec![
                line([0.0, 0.0], [30.0, 0.0]),
                Segment2::Arc {
                    center: [30.0, 5.0],
                    radius: 5.0,
                    start_angle: -PI / 2.0,
                    end_angle: PI / 2.0,
                },
                line([30.0, 10.0], [0.0, 10.0]),
                Segment2::Arc {
                    center: [0.0, 5.0],
                    radius: 5.0,
                    start_angle: PI / 2.0,
                    end_angle: 3.0 * PI / 2.0,
                },
            ],
        };
        let f = prepare_face(&Workplane::XY, &lp, &[]).unwrap();
        assert!((area(&f.loops[0]) - (300.0 + 25.0 * PI)).abs() < 1e-9);
        // Consecutive edges share exact endpoints after snapping.
        let e = &f.loops[0];
        for i in 0..e.len() {
            assert_eq!(e[i].to, e[(i + 1) % e.len()].from);
        }
    }

    #[test]
    fn gap_above_tolerance_is_rejected_and_below_is_snapped() {
        let mut lp = rect(0.0, 0.0, 1.0, 1.0);
        lp.segments[1] = line([1.0 + 5e-8, 0.0], [1.0, 1.0]);
        assert!(prepare_face(&Workplane::XY, &lp, &[]).is_ok());
        lp.segments[1] = line([1.0 + 2e-7, 0.0], [1.0, 1.0]);
        assert!(is_invalid(prepare_face(&Workplane::XY, &lp, &[])));
        // An open chain: the last segment doesn't return to the start.
        let mut open = rect(0.0, 0.0, 1.0, 1.0);
        open.segments[3] = line([0.0, 1.0], [0.0, 0.5]);
        assert!(is_invalid(prepare_face(&Workplane::XY, &open, &[])));
    }

    #[test]
    fn empty_and_degenerate_loops_are_rejected() {
        let wp = Workplane::XY;
        assert!(is_invalid(prepare_face(&wp, &ProfileLoop::default(), &[])));
        let r = rect(0.0, 0.0, 1.0, 1.0);
        assert!(is_invalid(prepare_face(&wp, &r, &[ProfileLoop::default()])));
        let single_line = ProfileLoop {
            segments: vec![line([0.0, 0.0], [1.0, 0.0])],
        };
        assert!(is_invalid(prepare_face(&wp, &single_line, &[])));
        let back_and_forth = ProfileLoop {
            segments: vec![line([0.0, 0.0], [1.0, 0.0]), line([1.0, 0.0], [0.0, 0.0])],
        };
        assert!(is_invalid(prepare_face(&wp, &back_and_forth, &[])));
        let mut circle_in_chain = r.clone();
        circle_in_chain.segments.push(Segment2::Circle {
            center: [0.0; 2],
            radius: 1.0,
        });
        assert!(is_invalid(prepare_face(&wp, &circle_in_chain, &[])));
    }

    #[test]
    fn non_finite_numbers_are_rejected() {
        let wp = Workplane::XY;
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut lp = rect(0.0, 0.0, 1.0, 1.0);
            lp.segments[0] = line([bad, 0.0], [1.0, 0.0]);
            assert!(is_invalid(prepare_face(&wp, &lp, &[])));
            assert!(is_invalid(prepare_face(&wp, &circle([0.0, bad], 1.0), &[])));
            let arc = ProfileLoop {
                segments: vec![
                    Segment2::Arc {
                        center: [0.0; 2],
                        radius: 1.0,
                        start_angle: 0.0,
                        end_angle: bad,
                    },
                    line([-1.0, 0.0], [1.0, 0.0]),
                ],
            };
            assert!(is_invalid(prepare_face(&wp, &arc, &[])));
            let origin = Workplane {
                origin: [0.0, bad, 0.0],
                ..wp
            };
            assert!(is_invalid(prepare_face(
                &origin,
                &circle([0.0; 2], 1.0),
                &[]
            )));
            let normal = Workplane {
                normal: [0.0, 0.0, bad],
                ..wp
            };
            assert!(is_invalid(prepare_face(
                &normal,
                &circle([0.0; 2], 1.0),
                &[]
            )));
        }
    }

    #[test]
    fn non_positive_radius_is_rejected() {
        for r in [0.0, -1.0] {
            assert!(is_invalid(prepare_face(
                &Workplane::XY,
                &circle([0.0; 2], r),
                &[]
            )));
            let arc = ProfileLoop {
                segments: vec![
                    Segment2::Arc {
                        center: [0.0; 2],
                        radius: r,
                        start_angle: 0.0,
                        end_angle: PI,
                    },
                    line([-1.0, 0.0], [1.0, 0.0]),
                ],
            };
            assert!(is_invalid(prepare_face(&Workplane::XY, &arc, &[])));
        }
    }

    #[test]
    fn workplane_axes_must_be_non_zero_and_perpendicular() {
        let c = circle([0.0; 2], 1.0);
        let zero_x = Workplane {
            x_dir: [0.0; 3],
            ..Workplane::XY
        };
        assert!(is_invalid(prepare_face(&zero_x, &c, &[])));
        let zero_n = Workplane {
            normal: [0.0; 3],
            ..Workplane::XY
        };
        assert!(is_invalid(prepare_face(&zero_n, &c, &[])));
        let skew = Workplane {
            normal: [1e-8, 0.0, 1.0],
            ..Workplane::XY
        };
        assert!(is_invalid(prepare_face(&skew, &c, &[])));
        // Unnormalized but perpendicular axes are fine and get normalized.
        let scaled = Workplane {
            origin: [1.0, 2.0, 3.0],
            x_dir: [0.0, 3.0, 0.0],
            normal: [0.0, 0.0, -2.0],
        };
        let f = prepare_face(&scaled, &c, &[]).unwrap();
        assert_eq!(
            f.plane_array(),
            [1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0]
        );
    }

    #[test]
    fn standard_workplanes_are_right_handed() {
        // normal × x_dir is the 2D +v axis: Y, Z and Z.
        let cross = |a: [f64; 3], b: [f64; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let v = |w: Workplane| cross(w.normal, w.x_dir);
        assert_eq!(v(Workplane::XY), [0.0, 1.0, 0.0]);
        assert_eq!(v(Workplane::XZ), [0.0, 0.0, 1.0]);
        assert_eq!(v(Workplane::YZ), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn revolve_inputs_are_validated() {
        let z = Axis {
            origin: [0.0; 3],
            dir: [0.0, 0.0, 2.0],
        };
        assert_eq!(
            prepare_revolve(z, 1.0),
            Ok(([0.0; 3], [0.0, 0.0, 1.0], 1.0))
        );
        // 2π itself, and 2π plus a hair, are a full turn.
        assert_eq!(prepare_revolve(z, TAU).unwrap().2, TAU);
        assert_eq!(prepare_revolve(z, TAU + 1e-13).unwrap().2, TAU);
        let bad_angles = [
            0.0,
            -0.0,
            -1.0,
            TAU + 1e-9,
            7.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ];
        for angle in bad_angles {
            let err = prepare_revolve(z, angle).unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("revolve angle")),
                "{angle}: {err}"
            );
        }
        for dir in [[0.0; 3], [f64::NAN, 0.0, 1.0], [0.0, f64::INFINITY, 0.0]] {
            let err = prepare_revolve(Axis { dir, ..z }, 1.0).unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("axis direction")),
                "{dir:?}: {err}"
            );
        }
        for origin in [[f64::NAN, 0.0, 0.0], [0.0, 0.0, f64::NEG_INFINITY]] {
            let err = prepare_revolve(Axis { origin, ..z }, 1.0).unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("axis origin")),
                "{origin:?}: {err}"
            );
        }
    }

    #[test]
    fn revolve_profile_checks_use_the_tolerance() {
        // The y = 0 plane; the z axis lies in it.
        let profile = |side_min, side_max| RevolveProfile {
            plane_point: [0.0; 3],
            normal: [0.0, -1.0, 0.0],
            side_min,
            side_max,
        };
        let z = [0.0, 0.0, 1.0];
        let tol = REVOLVE_AXIS_TOLERANCE;
        let ok = |origin, dir, p: RevolveProfile| check_revolve_profile(origin, dir, &p).is_ok();
        assert!(ok([0.0; 3], z, profile(0.0, 10.0)));
        assert!(ok([0.0; 3], z, profile(-10.0, 0.0)));
        assert!(ok([0.0; 3], z, profile(-0.5 * tol, 10.0)));
        assert!(ok([0.0, 0.5 * tol, 0.0], z, profile(10.0, 20.0)));
        assert!(!ok([0.0; 3], z, profile(-2.0 * tol, 10.0)));
        assert!(!ok([0.0, 2.0 * tol, 0.0], z, profile(10.0, 20.0)));
        let tilted = [0.0, (2.0 * tol).sin(), (2.0 * tol).cos()];
        assert!(!ok([0.0; 3], tilted, profile(10.0, 20.0)));
    }

    #[test]
    fn extent_offsets_and_invalid_lengths() {
        assert_eq!(Extent::Distance(10.0).offset_and_length(), Ok((0.0, 10.0)));
        assert_eq!(
            Extent::Distance(-10.0).offset_and_length(),
            Ok((-10.0, 10.0))
        );
        assert_eq!(
            Extent::Symmetric(10.0).offset_and_length(),
            Ok((-5.0, 10.0))
        );
        for bad in [0.0, -0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(Extent::Distance(bad).offset_and_length().is_err());
            assert!(Extent::Symmetric(bad).offset_and_length().is_err());
        }
        assert_eq!(
            Extent::FromTo(10.0, 30.0).offset_and_length(),
            Ok((10.0, 20.0))
        );
        assert_eq!(
            Extent::FromTo(5.0, -15.0).offset_and_length(),
            Ok((-15.0, 20.0))
        );
        for (a, b) in [(3.0, 3.0), (f64::NAN, 1.0), (0.0, f64::INFINITY)] {
            assert!(Extent::FromTo(a, b).offset_and_length().is_err());
        }
        let face = crate::SubShape {
            kind: crate::TopoKind::Face,
            index: 0,
        };
        for ext in [Extent::ToFace(face), Extent::BetweenFaces(face, face)] {
            let err = ext.offset_and_length().unwrap_err();
            assert!(err.to_string().contains("needs a target body"), "{err}");
        }
    }

    #[test]
    fn through_all_has_no_length_of_its_own() {
        for dir in [
            ThroughDirection::Normal,
            ThroughDirection::Reversed,
            ThroughDirection::Both,
        ] {
            let err = Extent::ThroughAll(dir).offset_and_length().unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("through-all")),
                "{dir:?}: {err}"
            );
        }
    }

    #[test]
    fn encoding_lists_loop_sizes_and_8_numbers_per_edge() {
        let f = prepare_face(
            &Workplane::XY,
            &rect(0.0, 0.0, 4.0, 2.0),
            &[circle([2.0, 1.0], 0.5)],
        )
        .unwrap();
        let (counts, edges) = f.encode();
        assert_eq!(counts, vec![4, 1]);
        assert_eq!(edges.len(), 5 * 8);
        assert_eq!(&edges[..5], &[0.0, 0.0, 0.0, 4.0, 0.0]);
        assert_eq!(edges[32], EdgeKind::CircleCw as i32 as f64);
        assert_eq!(&edges[37..], &[2.0, 1.0, 0.5]);
    }

    fn arc(center: [f64; 2], radius: f64, a0: f64, a1: f64) -> Segment2 {
        Segment2::Arc {
            center,
            radius,
            start_angle: a0,
            end_angle: a1,
        }
    }

    #[test]
    fn sweep_path_chains_in_order_and_snaps_joins() {
        // Line down, then a quarter arc; the arc is given ccw, so it is
        // entered at its start (π) where the line ends.
        let path = [
            line([0.0, 0.0], [0.0, -10.0]),
            arc([20.0, -10.0], 20.0, PI, 1.5 * PI),
        ];
        let p = prepare_path(&Workplane::XZ, &path).unwrap();
        let e = &p.loops[0];
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].from, [0.0, 0.0]);
        assert_eq!(e[0].to, e[1].from);
        assert!(dist(e[1].from, [0.0, -10.0]) < 1e-12);
        assert!(dist(e[1].to, [20.0, -30.0]) < 1e-12);
        assert_eq!(e[1].kind, EdgeKind::ArcCcw);
        // A second segment given backwards is reversed.
        let back = [line([0.0, 0.0], [5.0, 0.0]), line([9.0, 3.0], [5.0, 0.0])];
        let p = prepare_path(&Workplane::XY, &back).unwrap();
        assert_eq!(p.loops[0][1].from, [5.0, 0.0]);
        assert_eq!(p.loops[0][1].to, [9.0, 3.0]);
        // Workplane::XZ maps (u, v) to (u, 0, v).
        let q = p.point_3d([2.0, 3.0]);
        assert_eq!(
            prepare_path(&Workplane::XZ, &back)
                .unwrap()
                .point_3d([2.0, 3.0]),
            [2.0, 0.0, 3.0]
        );
        assert_eq!(q, [2.0, 3.0, 0.0]);
    }

    #[test]
    fn sweep_path_rejects_bad_inputs() {
        let ok = line([0.0, 0.0], [0.0, 30.0]);
        let bad = |plane: &Workplane, path: &[Segment2]| {
            assert!(
                matches!(prepare_path(plane, path), Err(KernelError::InvalidInput(_))),
                "expected InvalidInput for {path:?} on {plane:?}"
            );
        };
        let xz = &Workplane::XZ;
        bad(xz, &[]);
        bad(xz, &[circle([0.0, 0.0], 5.0).segments[0]]);
        bad(xz, &[ok, circle([0.0, 30.0], 5.0).segments[0]]);
        bad(xz, &[arc([1.0, 0.0], 0.0, PI, 1.5 * PI)]);
        bad(xz, &[arc([1.0, 0.0], -2.0, PI, 1.5 * PI)]);
        bad(xz, &[line([0.0, f64::NAN], [0.0, 30.0])]);
        bad(xz, &[line([0.0, 0.0], [f64::INFINITY, 30.0])]);
        bad(xz, &[ok, line([0.0, 30.1], [5.0, 40.0])]);
        let zero_x = Workplane {
            x_dir: [0.0; 3],
            ..Workplane::XZ
        };
        bad(&zero_x, &[ok]);
        let skew = Workplane {
            x_dir: [1.0, 1.0, 0.0],
            ..Workplane::XZ
        };
        bad(&skew, &[ok]);
        // A gap within the tolerance is accepted.
        assert!(prepare_path(xz, &[ok, line([0.0, 30.0 + 5e-8], [5.0, 40.0])]).is_ok());
    }
}
