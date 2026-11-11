//! Prints the metric coarse thread table: each size's pitch, minor diameter
//! and tap-drill diameter. Then looks up two sizes by name and shows the
//! errors a lookup can give. Needs no OpenCascade.
//!
//! Used by the user manual ("Making holes").
//!
//! Run: `cargo run -p swx-kernel --example thread_table`

use swx_kernel::{metric_coarse, METRIC_COARSE};

fn main() {
    println!("size   pitch  minor   tap drill");
    for t in METRIC_COARSE {
        println!(
            "{:<6} {:>5.2} {:>6.2} {:>6.2}",
            t.designation(),
            t.pitch,
            t.minor_diameter(),
            t.tap_drill_diameter()
        );
    }
    for name in ["M6", "m10"] {
        match metric_coarse(name) {
            Ok(t) => println!(
                "{name:?} -> {}, pitch {} mm, tap drill {} mm",
                t.designation(),
                t.pitch,
                t.tap_drill_diameter()
            ),
            Err(e) => println!("{name:?} -> error: {e}"),
        }
    }
    for name in ["M7", "6"] {
        match metric_coarse(name) {
            Ok(t) => println!("{name:?} -> {}", t.designation()),
            Err(e) => println!("{name:?} -> error: {e}"),
        }
    }
}
