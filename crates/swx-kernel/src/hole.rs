//! Hole inputs for [`Kernel::hole`](crate::Kernel::hole) and their checks.
//!
//! The checks are kernel-free, so they are tested without OCCT. A backend
//! builds the hole as one or two coaxial cylinders that it cuts from the
//! solid: the drilled hole and, for a counterbore, the wider recess at its
//! mouth.

use crate::{validate_point, validate_size, Axis, KernelError, Point3};

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
}

/// One cylinder of a hole's cutting tool: its radius and its length from the
/// start point along the drill direction. `length` is `None` for a through
/// hole, whose length the backend takes from the solid's size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HoleCylinder {
    pub radius: f64,
    pub length: Option<f64>,
}

/// A checked hole: start point, unit drill direction and the cylinders to cut.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HolePlan {
    pub origin: Point3,
    pub dir: [f64; 3],
    pub cylinders: Vec<HoleCylinder>,
}

/// Checks hole inputs and returns the cylinders to cut.
///
/// Every check is [`KernelError::InvalidInput`]: a diameter, blind depth,
/// `cb_diameter` or `cb_depth` that is not finite or ≤ 0; `cb_diameter` ≤
/// `diameter`; a blind depth ≤ `cb_depth`; a zero-length or non-finite
/// `at.dir`; a non-finite `at.origin`.
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
