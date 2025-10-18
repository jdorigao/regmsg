use std::{env, fs, path::Path, process::Command};

/// Build script for compiling the drmhook C library and linking it with the Rust project.
///
/// This script performs the following tasks:
/// 1. Compiles `lib/drmhook.c` into a shared library `libdrmhook.so`
/// 2. Handles libdrm dependency resolution via pkg-config
/// 3. Copies the compiled library to the target directory for easy access
/// 4. Sets up Cargo linking instructions
///
/// The build process uses the system C compiler and falls back to default paths
/// if pkg-config is unavailable or libdrm is not found.
fn main() {
    // Retrieve build environment variables set by Cargo
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let profile = env::var("PROFILE").expect("PROFILE environment variable not set");
    let lib_name = "libdrmhook.so";
    let out_lib = format!("{}/{}", out_dir, lib_name);

    // Verify that the C source file exists before attempting compilation
    let source_file = "lib/drmhook.c";
    if !Path::new(source_file).exists() {
        panic!("Source file not found: {}", source_file);
    }

    // Attempt to locate libdrm development files using pkg-config
    // This provides proper include paths and compilation flags
    let include_paths = match pkg_config::Config::new().probe("libdrm") {
        Ok(lib) => lib.include_paths,
        Err(_) => {
            // Fallback to default system include path if pkg-config fails
            vec!["/usr/include".into()]
        }
    };

    // Configure the compiler command with necessary flags:
    // - -shared: Create a shared library
    // - -fPIC: Generate position-independent code (required for shared libraries)
    // - -O2: Optimization level 2
    // - -ldrm: Link against libdrm
    // - -ldl: Link against libdl (dynamic loading library)
    let mut cmd = Command::new("cc");
    cmd.args([
        "-shared",
        "-fPIC",
        "-O2",
        source_file,
        "-o",
        &out_lib,
        "-ldrm",
        "-ldl",
    ]);

    // Add include paths discovered via pkg-config or fallback
    for path in &include_paths {
        cmd.arg("-I").arg(path);
    }

    // Execute the compilation command
    let status = cmd.status().expect("Failed to execute compiler");

    if !status.success() {
        // If the first attempt fails, try a simpler approach without optimization
        // This helps with compatibility on different systems
        let status_simple = Command::new("cc")
            .args([
                "-shared",
                "-fPIC",
                source_file,
                "-o",
                &out_lib,
                "-ldrm",
                "-ldl",
            ])
            .status()
            .expect("Failed to execute compiler (alternative approach)");

        if !status_simple.success() {
            panic!("Failed to compile libdrmhook.so - check compiler output for details");
        }
    }

    // Verify that the shared library was successfully created
    if Path::new(&out_lib).exists() {
        // Copy the library to the target directory for easier access during development
        let target_lib = format!("target/{}/{}", profile, lib_name);
        fs::create_dir_all(format!("target/{}", profile))
            .expect("Failed to create target directory");

        fs::copy(&out_lib, &target_lib).expect("Failed to copy library to target directory");
    } else {
        panic!("Library was not created at expected location: {}", out_lib);
    }

    // Tell Cargo where to find the compiled library
    println!("cargo:rustc-link-search=native={}", out_dir);

    // Tell Cargo to link against the drmhook library as a dynamic library
    println!("cargo:rustc-link-lib=dylib=drmhook");

    // Re-run this build script if the C source file changes
    println!("cargo:rerun-if-changed=lib/drmhook.c");

    // Re-run if pkg-config environment changes (could affect library discovery)
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    // Re-run if Cargo build profile changes (debug/release)
    println!("cargo:rerun-if-env-changed=PROFILE");
}
