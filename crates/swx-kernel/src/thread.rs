//! ISO metric coarse thread data for tapped holes. Pure data, no OCCT.
//!
//! Pitches are the coarse series of ISO 261 (ISO general purpose metric
//! screw threads, general plan). The basic minor diameter follows the basic
//! profile of ISO 68-1: with fundamental triangle height `H = (√3/2)·P`, the
//! internal thread's minor diameter is `D1 = D − 2·(5/8)·H = D − 1.082532·P`.
//! The tap-drill diameter uses the common workshop rule `D − P`.

use crate::KernelError;

/// One size of the ISO metric thread series: nominal (major) diameter and
/// pitch, both in mm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricThread {
    /// Nominal (major) diameter `D`, mm.
    pub nominal: f64,
    /// Pitch `P`, mm (ISO 261).
    pub pitch: f64,
}

impl MetricThread {
    /// The size's designation: `"M"` followed by the nominal diameter, e.g.
    /// `"M6"` or `"M2.5"`.
    pub fn designation(&self) -> String {
        format!("M{}", self.nominal)
    }

    /// Basic minor diameter of the internal thread, `D1 = D − 1.082532·P`
    /// (ISO 68-1 basic profile: `D1 = D − (5/4)·H` with `H = (√3/2)·P`).
    pub fn minor_diameter(&self) -> f64 {
        self.nominal - 1.082532 * self.pitch
    }

    /// Tap-drill diameter by the common rule `D − P`.
    pub fn tap_drill_diameter(&self) -> f64 {
        self.nominal - self.pitch
    }
}

const fn m(nominal: f64, pitch: f64) -> MetricThread {
    MetricThread { nominal, pitch }
}

/// The ISO 261 metric coarse series from M2 to M24, ascending by nominal
/// diameter.
pub const METRIC_COARSE: &[MetricThread] = &[
    m(2.0, 0.4),
    m(2.5, 0.45),
    m(3.0, 0.5),
    m(4.0, 0.7),
    m(5.0, 0.8),
    m(6.0, 1.0),
    m(8.0, 1.25),
    m(10.0, 1.5),
    m(12.0, 1.75),
    m(16.0, 2.0),
    m(20.0, 2.5),
    m(24.0, 3.0),
];

/// Looks up a size of [`METRIC_COARSE`] by designation, such as `"M6"`.
/// Upper or lower case `M` and surrounding whitespace are accepted.
///
/// It is [`KernelError::InvalidInput`] when the string is empty, is not `M`
/// followed by a decimal number (digits with at most one `.`), or names a
/// size that is not in the table.
pub fn metric_coarse(designation: &str) -> Result<MetricThread, KernelError> {
    let s = designation.trim();
    let number = s
        .strip_prefix('M')
        .or_else(|| s.strip_prefix('m'))
        .filter(|n| {
            !n.is_empty()
                && n.chars().all(|c| c.is_ascii_digit() || c == '.')
                && n.chars().filter(|&c| c == '.').count() <= 1
                && n.chars().any(|c| c.is_ascii_digit())
        })
        .ok_or_else(|| {
            KernelError::InvalidInput(format!(
                "thread designation {designation:?} must be M followed by a size, e.g. \"M6\""
            ))
        })?;
    let nominal: f64 = number.parse().map_err(|_| {
        KernelError::InvalidInput(format!(
            "thread designation {designation:?} has no valid size"
        ))
    })?;
    METRIC_COARSE
        .iter()
        .copied()
        .find(|t| t.nominal == nominal)
        .ok_or_else(|| {
            KernelError::InvalidInput(format!(
                "no metric coarse thread of size M{nominal} (sizes: M2 to M24)"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn check(designation: &str, pitch: f64, minor: f64, tap: f64) {
        let t = metric_coarse(designation).unwrap();
        assert!(close(t.pitch, pitch), "{designation}: {t:?}");
        assert!(close(t.minor_diameter(), minor), "{designation}: {t:?}");
        assert!(close(t.tap_drill_diameter(), tap), "{designation}: {t:?}");
    }

    #[test]
    fn lookups_give_pitch_minor_and_tap_drill() {
        check("M6", 1.0, 4.9175, 5.0);
        check("M3", 0.5, 2.4587, 2.5);
        check("m10 ", 1.5, 8.3762, 8.5);
        let t = metric_coarse("M2.5").unwrap();
        assert!(close(t.pitch, 0.45));
        assert_eq!(t.designation(), "M2.5");
        assert_eq!(metric_coarse("  M6\t").unwrap().designation(), "M6");
    }

    #[test]
    fn table_is_ascending_and_round_trips() {
        assert_eq!(METRIC_COARSE.len(), 12);
        for w in METRIC_COARSE.windows(2) {
            assert!(w[0].nominal < w[1].nominal, "{w:?}");
        }
        for t in METRIC_COARSE {
            assert_eq!(metric_coarse(&t.designation()).unwrap(), *t);
        }
    }

    #[test]
    fn bad_designations_are_invalid_input() {
        for bad in [
            "M7", "", "   ", "6", "Mx", "M-6", "M", "M.", "M6.0.0", "M 6", "MM6", "M+6", "Minf",
            "MNaN", "M6e0", "M100",
        ] {
            assert!(
                matches!(metric_coarse(bad), Err(KernelError::InvalidInput(_))),
                "{bad:?} should be InvalidInput"
            );
        }
    }
}
