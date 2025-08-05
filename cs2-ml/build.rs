fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=/Users/christian.klat/libtorch/libtorch/lib");

    // Tell cargo to tell rustc to link the system libraries
    println!("cargo:rustc-link-lib=torch");
    println!("cargo:rustc-link-lib=c10");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-env-changed=LIBTORCH");
}
