use std::{env, fs, path::Path};

/// Builds a shared library from C source code using the cc crate.
/// This build script compiles the drmhook.c file into a shared library (.so file) 
/// and makes it available for linking with the Rust application.
/// 
/// The script uses pkg-config to locate libdrm dependencies and their include paths.
/// It creates the shared library in the Cargo output directory and copies it to 
/// the appropriate target directory based on the build profile (debug/release).
/// 
/// The resulting library (libdrmhook.so) is linked dynamically during the Rust build process.
fn main() {
    // Retrieve the output directory from the OUT_DIR environment variable set by Cargo.
    // This directory is where build artifacts are placed during the build process.
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    // Define the name of the shared library to be created.
    // The library will be named "libdrmhook.so" and placed in the output directory.
    let lib_name = "libdrmhook.so";

    // Retrieve the build profile (e.g., debug or release) from the PROFILE environment variable.
    // This is used to determine the destination directory for the copied library.
    let profile = env::var("PROFILE").expect("PROFILE environment variable not set");

    // Initialize a new cc::Build instance to compile C code.
    // This builder will be configured with appropriate flags and include paths.
    let mut build = cc::Build::new();

    // Configure the build to compile the drmhook.c source file
    // and enable position-independent code which is required for shared libraries.
    build
        .file("lib/drmhook.c")
        .pic(true);

    // Attempt to use pkg-config to locate libdrm and its include paths.
    // This ensures the C code can find required headers like drm.h and xf86drm.h
    match pkg_config::Config::new()
        .env_metadata(true) // Include environment metadata in the probe to allow environment override.
        .cargo_metadata(false) // Disable automatic Cargo metadata output to avoid interference.
        .probe("libdrm") // Probe for the libdrm library and its configuration.
    {
        Ok(lib) => {
            // If libdrm is found successfully, include all its include paths
            // in the build configuration to ensure proper compilation.
            for path in lib.include_paths {
                build.include(path);
            }
        }
        Err(_) => {
            // If pkg-config fails to locate libdrm, fall back to a default include path.
            // This provides basic functionality when pkg-config is not available or libdrm
            // is installed in a standard location.
            eprintln!("⚠️ pkg-config failed, falling back to default include path");
            build.include("/usr/include");
        }
    }

    // Compile the C code with proper flags for shared library generation.
    // This step generates the object files needed for the shared library.
    let objects = build.compile_intermediates();
    
    // Create a shared library from the compiled object file using the compiler directly.
    // The cc crate primarily creates static libraries, so we manually create a shared library.
    
    // Get the compiler instance from the cc::Build configuration
    // to ensure consistent flags and settings are used.
    let compiler = build.get_compiler();
    let so_file = format!("{}/libdrmhook.so", out_dir);

    // Execute the compiler command to create a shared library with the required flags:
    // -shared: Create a shared library
    // -fPIC: Generate position-independent code (required for shared libraries)
    // -ldrm: Link against the libdrm library
    // -ldl: Link against the dynamic linking library
    let mut cmd = compiler.to_command();
    cmd.arg("-shared")
        .arg("-fPIC")
        .arg(&objects[0])  // Use the compiled object file from the cc build
        .arg("-o")
        .arg(&so_file)
        .arg("-ldrm")
        .arg("-ldl");

    // Execute the command to create the shared library
    cmd.status().expect("Failed to create shared library");

    // Construct the path to the compiled library in the output directory.
    let lib_path = Path::new(&out_dir).join(lib_name);

    // Construct the path where the library will be copied for easier access.
    // This copies the library to a standard location in the target directory.
    let final_copy_path = format!("target/{}/{}", profile, lib_name);

    // Create the target/profile directory if it doesn't exist.
    // This ensures the destination directory exists before attempting to copy.
    fs::create_dir_all(format!("target/{}", profile)).expect("Failed to create destination directory");

    // Copy the compiled library to the target/profile directory for debugging or manual access.
    // This provides easy access to the generated library file for external tools or inspection.
    fs::copy(&lib_path, &final_copy_path).expect("Failed to copy library to destination directory");

    // Instruct Cargo to add the output directory to the native library search paths.
    // This allows the Rust linker to find the generated library during the linking phase.
    println!("cargo:rustc-link-search=native={}", out_dir);

    // Instruct Cargo to link the drmhook library as a dynamic library.
    // This tells the Rust linker to link against libdrmhook.so during compilation.
    println!("cargo:rustc-link-lib=dylib=drmhook");

    // Inform Cargo to re-run this build script if the drmhook.c source file changes.
    // This ensures the C library is rebuilt when the source code is modified.
    println!("cargo:rerun-if-changed=lib/drmhook.c");
}
