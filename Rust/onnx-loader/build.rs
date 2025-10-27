// Build script to resolve Windows C runtime mismatch
// Suppresses LIBCMT (static runtime) conflict with ort's dynamic runtime

fn main() {
    if cfg!(target_os = "windows") {
        // Tell linker to ignore the static C runtime library
        // This allows esaxx_rs (static) and ort_sys (dynamic) to coexist
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt");
    }
}

