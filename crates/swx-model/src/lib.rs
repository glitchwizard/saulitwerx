//! Feature evaluation for Saulitwerx: the code that joins sketches
//! (`swx-sketch`) to the geometry kernel (`swx-kernel`).
//!
//! It sits above both crates (`model → kernel, sketch → core`), so neither
//! of them needs to know about the other. See the ADR log in
//! `docs/ARCHITECTURE.md` (M3-01c) and `docs/specs/M3-04.md` ("Data model").
//!
//! Today it owns one conversion: a sketch [`Region`](swx_sketch::profile::Region)
//! to the [`ProfileLoop`](swx_kernel::ProfileLoop)s that
//! [`Kernel::make_face`](swx_kernel::Kernel::make_face) takes, remembering the
//! sketch entity behind every segment ([`region_to_profile`]). The naming
//! table and the topological-reference resolver (M3-04f) come next.

pub mod profile;

pub use profile::{region_to_profile, ModelError, RegionProfile};
