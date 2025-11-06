fn main() {
    // Link Windows system libraries that mdbx-sys needs but doesn't properly declare
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
    }
}

