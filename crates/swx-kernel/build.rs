//! Build script for `swx-kernel`.
//!
//! With the `occt` feature it compiles `cpp/occt_shim.cpp` (a C ABI wrapper
//! around OCCT) and links the OCCT toolkits dynamically. Without the feature
//! it does nothing, so the default build needs no C++ toolchain or OCCT.
//!
//! Environment overrides:
//! - `OCCT_INCLUDE_DIR` (default `/usr/include/opencascade`)
//! - `OCCT_LIB_DIR` (default: the system linker search path)

fn main() {
    #[cfg(feature = "occt")]
    occt::build();
}

#[cfg(feature = "occt")]
mod occt {
    /// OCCT toolkits needed for primitives, booleans, bounding boxes and mass
    /// properties, in link order.
    const TOOLKITS: &[&str] = &[
        "TKBO",
        "TKPrim",
        "TKTopAlgo",
        "TKGeomAlgo",
        "TKBRep",
        "TKGeomBase",
        "TKG3d",
        "TKG2d",
        "TKMath",
        "TKernel",
    ];

    pub fn build() {
        println!("cargo:rerun-if-changed=cpp/occt_shim.cpp");
        println!("cargo:rerun-if-env-changed=OCCT_INCLUDE_DIR");
        println!("cargo:rerun-if-env-changed=OCCT_LIB_DIR");

        let include = std::env::var("OCCT_INCLUDE_DIR")
            .unwrap_or_else(|_| "/usr/include/opencascade".to_string());

        cc::Build::new()
            .cpp(true)
            .std("c++17")
            .include(&include)
            // OCCT 7.6 headers trip libstdc++ C++17 deprecation warnings.
            .flag_if_supported("-Wno-deprecated-declarations")
            .file("cpp/occt_shim.cpp")
            .compile("swx_occt_shim");

        if let Ok(dir) = std::env::var("OCCT_LIB_DIR") {
            println!("cargo:rustc-link-search=native={dir}");
        }
        // `dylib` keeps OCCT (LGPL-2.1 with exception) a separate shared library.
        for tk in TOOLKITS {
            println!("cargo:rustc-link-lib=dylib={tk}");
        }
    }
}
