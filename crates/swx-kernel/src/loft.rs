//! Kernel-free input checks for [`Kernel::loft`](crate::Kernel::loft).

use crate::KernelError;

/// A loft needs at least this many sections.
pub const MIN_LOFT_SECTIONS: usize = 2;

/// `InvalidInput` when there are fewer than [`MIN_LOFT_SECTIONS`] sections.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn check_section_count(n: usize) -> Result<(), KernelError> {
    if n < MIN_LOFT_SECTIONS {
        return Err(KernelError::InvalidInput(format!(
            "a loft needs at least {MIN_LOFT_SECTIONS} sections, got {n}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_loft_needs_two_or_more_sections() {
        for n in [0, 1] {
            assert!(matches!(
                check_section_count(n),
                Err(KernelError::InvalidInput(_))
            ));
        }
        for n in [2, 3, 10] {
            assert!(check_section_count(n).is_ok());
        }
    }
}
