//! Input checks for [`Kernel::split`] (M6-02a). A plain function, so it is
//! tested without OCCT; the backend only intersects the shape with each
//! half-space.
//!
//! [`Kernel::split`]: crate::Kernel::split

use crate::profile::{finite, normalized};
use crate::{KernelError, Point3, Workplane};

/// The split plane's origin and unit normal, from `plane.origin` and
/// `plane.normal` (`plane.x_dir` is not used). A non-finite origin, or a
/// zero or non-finite normal, is `InvalidInput`.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn prepare_split(plane: &Workplane) -> Result<(Point3, [f64; 3]), KernelError> {
    finite("split plane origin", &plane.origin)?;
    let normal = normalized("split plane normal", plane.normal)?;
    Ok((plane.origin, normal))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_invalid<T: std::fmt::Debug>(r: Result<T, KernelError>) -> bool {
        matches!(r, Err(KernelError::InvalidInput(_)))
    }

    #[test]
    fn split_plane_is_normalized_and_bad_planes_are_rejected() {
        let plane = Workplane {
            origin: [4.0, 0.0, 0.0],
            // Not used, so even a zero x_dir is accepted.
            x_dir: [0.0; 3],
            normal: [-3.0, 0.0, 0.0],
        };
        assert_eq!(
            prepare_split(&plane).unwrap(),
            ([4.0, 0.0, 0.0], [-1.0, 0.0, 0.0])
        );
        for normal in [
            [0.0; 3],
            [f64::NAN, 0.0, 1.0],
            [0.0, f64::INFINITY, 0.0],
            [0.0, 0.0, f64::NEG_INFINITY],
        ] {
            assert!(is_invalid(prepare_split(&Workplane { normal, ..plane })));
        }
        for origin in [
            [f64::NAN, 0.0, 0.0],
            [0.0, f64::INFINITY, 0.0],
            [0.0, 0.0, f64::NEG_INFINITY],
        ] {
            assert!(is_invalid(prepare_split(&Workplane { origin, ..plane })));
        }
    }
}
