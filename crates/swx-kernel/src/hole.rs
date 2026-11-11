//! Hole inputs for [`Kernel::hole`](crate::Kernel::hole) and their checks.
//!
//! The checks are kernel-free, so they are tested without OCCT. A backend
//! builds the hole as one or two coaxial cylinders that it cuts from the
//! solid: the drilled hole and, for a counterbore, the wider recess at its
//! mouth. A countersink adds a coaxial cone (a frustum) at the mouth instead.

use crate::{validate_point, validate_size, Axis, KernelError, MetricThread, Point3};

/// How deep a hole goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoleDepth {
    /// Depth from the hole's start point along the drill direction to its
    /// flat bottom (no drill point).
    Blind(f64),
    /// Through the whole solid along the drill direction from the start point.
    ThroughAll,
}

/// The shape of a hole's cross-section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoleKind {
    /// A plain cylindrical hole.
    Simple {
        /// Hole diameter.
        diameter: f64,
    },
    /// A cylindrical hole with a wider, flat-bottomed recess at its mouth.
    Counterbore {
        /// Hole diameter.
        diameter: f64,
        /// Recess diameter; larger than `diameter`.
        cb_diameter: f64,
        /// Recess depth, measured from the start point along the drill
        /// direction, like a blind depth.
        cb_depth: f64,
    },
    /// A cylindrical hole with a conical recess at its mouth. The cone runs
    /// from `csk_diameter` at the start point down to `diameter`, so its
    /// depth along the drill direction is
    /// `(csk_diameter − diameter) / 2 / tan(csk_angle / 2)`.
    Countersink {
        /// Hole diameter.
        diameter: f64,
        /// Cone diameter at the start point; larger than `diameter`.
        csk_diameter: f64,
        /// Included cone angle in radians, strictly between 0 and π
        /// (e.g. π/2 for a 90° countersink).
        csk_angle: f64,
    },
    /// A hole for a metric coarse thread. The cut is a plain cylinder of the
    /// thread's tap-drill diameter ([`MetricThread::tap_drill_diameter`],
    /// `D − P`); the thread itself is not modeled.
    Tapped {
        /// The thread size, e.g. from [`metric_coarse`](crate::metric_coarse).
        thread: MetricThread,
    },
}

/// The conical recess of a countersink: a frustum on the drill axis from
/// `top_radius` at the start point to `bottom_radius` at `length` along the
/// drill direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HoleCone {
    pub top_radius: f64,
    pub bottom_radius: f64,
    pub length: f64,
}

/// One cylinder of a hole's cutting tool: its radius and its length from the
/// start point along the drill direction. `length` is `None` for a through
/// hole, whose length the backend takes from the solid's size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HoleCylinder {
    pub radius: f64,
    pub length: Option<f64>,
}

/// A checked hole: start point, unit drill direction, the cylinders to cut
/// and, for a countersink, the cone to cut.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HolePlan {
    pub origin: Point3,
    pub dir: [f64; 3],
    pub cylinders: Vec<HoleCylinder>,
    pub cone: Option<HoleCone>,
}

/// Checks hole inputs and returns the cylinders (and cone) to cut.
///
/// Every check is [`KernelError::InvalidInput`]: a diameter, blind depth,
/// `cb_diameter`, `cb_depth`, `csk_diameter`, thread `nominal` or thread
/// `pitch` that is not finite or ≤ 0; a thread `pitch` ≥ its `nominal`;
/// `cb_diameter` or `csk_diameter` ≤ `diameter`; a `csk_angle` that is not
/// finite or not strictly between 0 and π; a blind depth ≤ `cb_depth` or ≤
/// the countersink's cone depth; a zero-length or non-finite `at.dir`; a
/// non-finite `at.origin`.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn prepare_hole(
    at: Axis,
    kind: HoleKind,
    depth: HoleDepth,
) -> Result<HolePlan, KernelError> {
    let blind = match depth {
        HoleDepth::Blind(d) => {
            validate_size("hole depth", d)?;
            Some(d)
        }
        HoleDepth::ThroughAll => None,
    };
    let mut cylinders = Vec::with_capacity(2);
    let mut cone = None;
    match kind {
        HoleKind::Simple { diameter } => {
            validate_size("hole diameter", diameter)?;
            cylinders.push(HoleCylinder {
                radius: diameter / 2.0,
                length: blind,
            });
        }
        HoleKind::Counterbore {
            diameter,
            cb_diameter,
            cb_depth,
        } => {
            validate_size("hole diameter", diameter)?;
            validate_size("counterbore diameter", cb_diameter)?;
            validate_size("counterbore depth", cb_depth)?;
            if cb_diameter <= diameter {
                return Err(KernelError::InvalidInput(format!(
                    "counterbore diameter {cb_diameter} must be larger than the hole diameter {diameter}"
                )));
            }
            if let Some(d) = blind {
                if d <= cb_depth {
                    return Err(KernelError::InvalidInput(format!(
                        "hole depth {d} must be larger than the counterbore depth {cb_depth}"
                    )));
                }
            }
            cylinders.push(HoleCylinder {
                radius: diameter / 2.0,
                length: blind,
            });
            cylinders.push(HoleCylinder {
                radius: cb_diameter / 2.0,
                length: Some(cb_depth),
            });
        }
        HoleKind::Countersink {
            diameter,
            csk_diameter,
            csk_angle,
        } => {
            validate_size("hole diameter", diameter)?;
            validate_size("countersink diameter", csk_diameter)?;
            if !csk_angle.is_finite() || csk_angle <= 0.0 || csk_angle >= std::f64::consts::PI {
                return Err(KernelError::InvalidInput(format!(
                    "countersink angle {csk_angle} must be strictly between 0 and π radians"
                )));
            }
            if csk_diameter <= diameter {
                return Err(KernelError::InvalidInput(format!(
                    "countersink diameter {csk_diameter} must be larger than the hole diameter {diameter}"
                )));
            }
            let cone_depth = (csk_diameter - diameter) / 2.0 / (csk_angle / 2.0).tan();
            if let Some(d) = blind {
                if d <= cone_depth {
                    return Err(KernelError::InvalidInput(format!(
                        "hole depth {d} must be larger than the countersink depth {cone_depth}"
                    )));
                }
            }
            cylinders.push(HoleCylinder {
                radius: diameter / 2.0,
                length: blind,
            });
            cone = Some(HoleCone {
                top_radius: csk_diameter / 2.0,
                bottom_radius: diameter / 2.0,
                length: cone_depth,
            });
        }
        HoleKind::Tapped { thread } => {
            validate_size("thread nominal diameter", thread.nominal)?;
            validate_size("thread pitch", thread.pitch)?;
            if thread.pitch >= thread.nominal {
                return Err(KernelError::InvalidInput(format!(
                    "thread pitch {} must be smaller than the nominal diameter {}",
                    thread.pitch, thread.nominal
                )));
            }
            cylinders.push(HoleCylinder {
                radius: thread.tap_drill_diameter() / 2.0,
                length: blind,
            });
        }
    }
    let [x, y, z] = at.dir;
    let len = (x * x + y * y + z * z).sqrt();
    if !len.is_finite() || len == 0.0 {
        return Err(KernelError::InvalidInput(format!(
            "hole direction must be finite and non-zero, got {:?}",
            at.dir
        )));
    }
    validate_point("hole start point", at.origin)?;
    Ok(HolePlan {
        origin: at.origin,
        dir: [x / len, y / len, z / len],
        cylinders,
        cone,
    })
}

/// A unit vector perpendicular to the unit vector `n`: `n` crossed with the
/// coordinate axis it is least aligned with, normalized.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn perpendicular(n: [f64; 3]) -> [f64; 3] {
    let a = n.map(f64::abs);
    let e = if a[0] <= a[1] && a[0] <= a[2] {
        [1.0, 0.0, 0.0]
    } else if a[1] <= a[2] {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let c = [
        n[1] * e[2] - n[2] * e[1],
        n[2] * e[0] - n[0] * e[2],
        n[0] * e[1] - n[1] * e[0],
    ];
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    c.map(|v| v / len)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOWN: Axis = Axis {
        origin: [20.0, 15.0, 20.0],
        dir: [0.0, 0.0, -1.0],
    };

    fn simple(diameter: f64) -> HoleKind {
        HoleKind::Simple { diameter }
    }

    fn cbore(diameter: f64, cb_diameter: f64, cb_depth: f64) -> HoleKind {
        HoleKind::Counterbore {
            diameter,
            cb_diameter,
            cb_depth,
        }
    }

    fn assert_invalid(at: Axis, kind: HoleKind, depth: HoleDepth) {
        assert!(
            matches!(
                prepare_hole(at, kind, depth),
                Err(KernelError::InvalidInput(_))
            ),
            "{at:?} {kind:?} {depth:?} should be InvalidInput"
        );
    }

    #[test]
    fn valid_holes_give_their_cylinders() {
        let at = Axis {
            origin: [1.0, 2.0, 3.0],
            dir: [0.0, 0.0, -2.0],
        };
        let p = prepare_hole(at, simple(6.0), HoleDepth::Blind(10.0)).unwrap();
        assert_eq!(p.origin, [1.0, 2.0, 3.0]);
        assert_eq!(p.dir, [0.0, 0.0, -1.0]);
        assert_eq!(
            p.cylinders,
            vec![HoleCylinder {
                radius: 3.0,
                length: Some(10.0)
            }]
        );
        let p = prepare_hole(at, cbore(6.0, 11.0, 6.0), HoleDepth::ThroughAll).unwrap();
        assert_eq!(
            p.cylinders,
            vec![
                HoleCylinder {
                    radius: 3.0,
                    length: None
                },
                HoleCylinder {
                    radius: 5.5,
                    length: Some(6.0)
                }
            ]
        );
    }

    #[test]
    fn bad_sizes_are_invalid_input() {
        let blind = HoleDepth::Blind(10.0);
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_invalid(DOWN, simple(bad), blind);
            assert_invalid(DOWN, simple(6.0), HoleDepth::Blind(bad));
            assert_invalid(DOWN, cbore(bad, 11.0, 6.0), blind);
            assert_invalid(DOWN, cbore(6.0, bad, 6.0), blind);
            assert_invalid(DOWN, cbore(6.0, 11.0, bad), blind);
            assert_invalid(DOWN, cbore(6.0, 11.0, bad), HoleDepth::ThroughAll);
        }
    }

    #[test]
    fn counterbore_must_be_wider_and_shallower() {
        assert_invalid(DOWN, cbore(6.0, 5.0, 6.0), HoleDepth::ThroughAll);
        assert_invalid(DOWN, cbore(6.0, 6.0, 6.0), HoleDepth::ThroughAll);
        assert_invalid(DOWN, cbore(6.0, 11.0, 6.0), HoleDepth::Blind(6.0));
        assert_invalid(DOWN, cbore(6.0, 11.0, 6.0), HoleDepth::Blind(4.0));
        assert!(prepare_hole(DOWN, cbore(6.0, 11.0, 6.0), HoleDepth::Blind(6.5)).is_ok());
    }

    fn csk(diameter: f64, csk_diameter: f64, csk_angle: f64) -> HoleKind {
        HoleKind::Countersink {
            diameter,
            csk_diameter,
            csk_angle,
        }
    }

    const RIGHT: f64 = std::f64::consts::FRAC_PI_2;

    #[test]
    fn countersink_gives_its_cylinder_and_cone() {
        let p = prepare_hole(DOWN, csk(6.0, 12.0, RIGHT), HoleDepth::Blind(10.0)).unwrap();
        assert_eq!(
            p.cylinders,
            vec![HoleCylinder {
                radius: 3.0,
                length: Some(10.0)
            }]
        );
        let cone = p.cone.unwrap();
        assert_eq!(cone.top_radius, 6.0);
        assert_eq!(cone.bottom_radius, 3.0);
        assert!((cone.length - 3.0).abs() < 1e-12, "{cone:?}");
        // 60° included angle: depth = 3 / tan(30°) = 3√3.
        let p = prepare_hole(
            DOWN,
            csk(6.0, 12.0, RIGHT * 2.0 / 3.0),
            HoleDepth::ThroughAll,
        )
        .unwrap();
        assert_eq!(p.cylinders[0].length, None);
        assert!((p.cone.unwrap().length - 3.0 * 3f64.sqrt()).abs() < 1e-12);
        // Other kinds have no cone.
        let p = prepare_hole(DOWN, cbore(6.0, 11.0, 6.0), HoleDepth::ThroughAll).unwrap();
        assert_eq!(p.cone, None);
    }

    #[test]
    fn countersink_bad_inputs_are_invalid_input() {
        let blind = HoleDepth::Blind(10.0);
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_invalid(DOWN, csk(bad, 12.0, RIGHT), blind);
            assert_invalid(DOWN, csk(6.0, bad, RIGHT), blind);
            assert_invalid(DOWN, csk(6.0, 12.0, bad), HoleDepth::ThroughAll);
        }
        let pi = std::f64::consts::PI;
        for bad in [pi, 4.0, -pi] {
            assert_invalid(DOWN, csk(6.0, 12.0, bad), HoleDepth::ThroughAll);
        }
        assert_invalid(DOWN, csk(6.0, 6.0, RIGHT), HoleDepth::ThroughAll);
        assert_invalid(DOWN, csk(6.0, 5.0, RIGHT), HoleDepth::ThroughAll);
        // Cone depth is 3: the blind depth must be larger.
        assert_invalid(DOWN, csk(6.0, 12.0, RIGHT), HoleDepth::Blind(3.0));
        assert_invalid(DOWN, csk(6.0, 12.0, RIGHT), HoleDepth::Blind(2.0));
        assert!(prepare_hole(DOWN, csk(6.0, 12.0, RIGHT), HoleDepth::Blind(3.5)).is_ok());
    }

    fn tapped(nominal: f64, pitch: f64) -> HoleKind {
        HoleKind::Tapped {
            thread: MetricThread { nominal, pitch },
        }
    }

    #[test]
    fn tapped_cuts_the_tap_drill_diameter() {
        let m6 = HoleKind::Tapped {
            thread: crate::metric_coarse("M6").unwrap(),
        };
        for depth in [HoleDepth::Blind(10.0), HoleDepth::ThroughAll] {
            let p = prepare_hole(DOWN, m6, depth).unwrap();
            assert_eq!(p, prepare_hole(DOWN, simple(5.0), depth).unwrap());
        }
    }

    #[test]
    fn tapped_bad_threads_are_invalid_input() {
        let blind = HoleDepth::Blind(10.0);
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_invalid(DOWN, tapped(bad, 1.0), blind);
            assert_invalid(DOWN, tapped(6.0, bad), blind);
            assert_invalid(DOWN, tapped(bad, 1.0), HoleDepth::ThroughAll);
        }
        assert_invalid(DOWN, tapped(6.0, 6.0), blind);
        assert_invalid(DOWN, tapped(6.0, 7.0), HoleDepth::ThroughAll);
        assert!(prepare_hole(DOWN, tapped(6.0, 5.9), blind).is_ok());
    }

    #[test]
    fn bad_axis_is_invalid_input() {
        let blind = HoleDepth::Blind(10.0);
        for dir in [
            [0.0, 0.0, 0.0],
            [f64::NAN, 0.0, -1.0],
            [0.0, f64::INFINITY, 0.0],
        ] {
            assert_invalid(Axis { dir, ..DOWN }, simple(6.0), blind);
        }
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let at = Axis {
                origin: [20.0, bad, 20.0],
                ..DOWN
            };
            assert_invalid(at, simple(6.0), blind);
        }
    }

    #[test]
    fn perpendicular_is_unit_and_orthogonal() {
        let s = 3f64.sqrt().recip();
        for n in [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            [s, s, s],
        ] {
            let p = perpendicular(n);
            let dot = p[0] * n[0] + p[1] * n[1] + p[2] * n[2];
            let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(dot.abs() < 1e-12, "{n:?} -> {p:?}");
            assert!((len - 1.0).abs() < 1e-12, "{n:?} -> {p:?}");
        }
    }
}
