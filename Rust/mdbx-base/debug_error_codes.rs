// Quick debug: Print ALL MDBX error constants
use mdbx_sys::*;

fn main() {
    println!("MDBX Error Codes:");
    println!("MDBX_SUCCESS = {}", MDBX_SUCCESS);
    println!("MDBX_NOTFOUND = {}", MDBX_NOTFOUND);
    
    // Print all constants we can find
    unsafe {
        // Try to find what -30791 actually is
        println!("\n-30791 might be:");
        println!("Checking common error codes...");
    }
}

