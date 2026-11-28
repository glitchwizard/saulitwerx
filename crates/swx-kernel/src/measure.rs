//! Measurement results: the minimum distance between two shapes or
//! sub-shapes (roadmap M6-04b), and the angle math behind
//! [`Kernel::angle`](crate::Kernel::angle) (M6-04c).

use crate::Point3;

/// The minimum distance between two shapes, from
/// [`Kernel::min_distance`](crate::Kernel::min_distance), with one pair of
/// closest points.
///
/// Lengths are in model units (mm). `on_a` lies on the first input and
/// `on_b` on the second, and |`on_a` − `on_b`| = `value`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Distance {
    /// The minimum distance, ≥ 0; 0 when the inputs touch or overlap.
    pub value: f64,
    /// A closest point on the first input.
    pub on_a: Point3,
    /// A closest point on the second input.
    pub on_b: Point3,
}

/// The direction a sub-shape contributes to [`Kernel::angle`](crate::Kernel::angle).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Direction {
    /// A planar face's unit outward normal.
    Normal([f64; 3]),
    /// A unit line direction (straight edge or cylinder axis), either sign.
    Line([f64; 3]),
}

/// The angle in radians between two directions, both unit vectors:
///
/// - normal–normal: the angle between the normals, `acos(n₁·n₂)`, in [0, π];
/// - line–line: the angle between the lines, `acos(|d₁·d₂|)`, in [0, π/2];
/// - normal–line (either order): the angle between the line and the plane,
///   `asin(|n·d|)`, in [0, π/2].
///
/// These are the usual definitions of the angle between two planes, two
/// lines, and a line and a plane in analytic geometry (e.g. Weisstein,
/// *MathWorld*, "Plane", "Line-Line Angle"). The dot product is clamped to
/// [−1, 1] first, so rounding past ±1 gives 0 or π (or π/2), never NaN.
pub(crate) fn angle_between(a: Direction, b: Direction) -> f64 {
    fn dot(u: [f64; 3], v: [f64; 3]) -> f64 {
        (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]).clamp(-1.0, 1.0)
    }
    match (a, b) {
        (Direction::Normal(n1), Direction::Normal(n2)) => dot(n1, n2).acos(),
        (Direction::Line(d1), Direction::Line(d2)) => dot(d1, d2).abs().acos(),
        (Direction::Normal(n), Direction::Line(d)) | (Direction::Line(d), Direction::Normal(n)) => {
            dot(n, d).abs().asin()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{angle_between, Direction};
    use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4, PI};

    const TOL: f64 = 1e-12;

    fn close(got: f64, want: f64) {
        assert!((got - want).abs() <= TOL, "got {got}, want {want}");
    }

    #[test]
    fn face_face_is_angle_between_normals() {
        let up = Direction::Normal([0.0, 0.0, 1.0]);
        let down = Direction::Normal([0.0, 0.0, -1.0]);
        let side = Direction::Normal([1.0, 0.0, 0.0]);
        let chamfer = Direction::Normal([0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2]);
        let front = Direction::Normal([0.0, -1.0, 0.0]);
        close(angle_between(up, up), 0.0);
        close(angle_between(up, down), PI);
        close(angle_between(up, side), FRAC_PI_2);
        close(angle_between(chamfer, up), FRAC_PI_4);
        close(angle_between(chamfer, front), 3.0 * FRAC_PI_4);
    }

    #[test]
    fn line_line_ignores_sign_and_is_at_most_right_angle() {
        let x = Direction::Line([1.0, 0.0, 0.0]);
        let minus_x = Direction::Line([-1.0, 0.0, 0.0]);
        let y = Direction::Line([0.0, 1.0, 0.0]);
        let diag = Direction::Line([-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0]);
        close(angle_between(x, minus_x), 0.0);
        close(angle_between(x, y), FRAC_PI_2);
        close(angle_between(x, diag), FRAC_PI_4);
    }

    #[test]
    fn face_line_is_angle_to_plane_in_both_orders() {
        let top = Direction::Normal([0.0, 0.0, 1.0]);
        let x = Direction::Line([1.0, 0.0, 0.0]);
        let z = Direction::Line([0.0, 0.0, -1.0]);
        let tilted = Direction::Line([FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2]);
        for (a, b) in [(top, x), (x, top)] {
            close(angle_between(a, b), 0.0);
        }
        for (a, b) in [(top, z), (z, top)] {
            close(angle_between(a, b), FRAC_PI_2);
        }
        for (a, b) in [(top, tilted), (tilted, top)] {
            close(angle_between(a, b), FRAC_PI_4);
        }
    }

    #[test]
    fn dot_rounding_past_one_is_clamped() {
        let over = [1.0 + f64::EPSILON, 0.0, 0.0];
        let x = [1.0, 0.0, 0.0];
        let minus_x = [-1.0, 0.0, 0.0];
        assert!(over[0] * x[0] > 1.0);
        let cases = [
            (Direction::Normal(over), Direction::Normal(x), 0.0),
            (Direction::Normal(over), Direction::Normal(minus_x), PI),
            (Direction::Line(over), Direction::Line(minus_x), 0.0),
            (Direction::Normal(over), Direction::Line(minus_x), FRAC_PI_2),
            (Direction::Line(over), Direction::Normal(x), FRAC_PI_2),
        ];
        for (a, b, want) in cases {
            let got = angle_between(a, b);
            assert!(got.is_finite(), "{a:?} {b:?} gave {got}");
            assert_eq!(got, want, "{a:?} {b:?}");
        }
    }
}
