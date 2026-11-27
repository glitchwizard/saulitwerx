//! Measurement results (roadmap M6-04b): the minimum distance between two
//! shapes or sub-shapes.

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
