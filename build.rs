use std::{env, fs, path::Path};

/// Main function to orchestrate the build process for the drmhook library.
/// This build script is used by Cargo to compile native C code and link the resulting library.
fn main() {
    // Retrieve the output directory from the OUT_DIR environment variable set by Cargo.
    // This directory is where build artifacts are placed.
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    // Define the name of the shared library to be created.
    // The library will be named "libdrmhook.so" and placed in the output directory.
    let lib_name = "libdrmhook.so";
    let out_lib = format!("{}/{}", out_dir, lib_name);

    // Retrieve the build profile (e.g., debug or release) from the PROFILE environment variable.
    let profile = env::var("PROFILE").expect("PROFILE environment variable not set");

    // Initialize a new cc::Build instance to compile C code.
    let mut build = cc::Build::new();

    // Compile intermediate object files from the C source (if any).
    let objects = build.compile_intermediates();

    // Attempt to use pkg-config to locate libdrm and its include paths.
    match pkg_config::Config::new()
        .env_metadata(true) // Include environment metadata in the probe.
        .cargo_metadata(false) // Disable Cargo metadata output.
        .probe("libdrm") // Probe for the libdrm library.
    {
        Ok(lib) => {
            // If libdrm is found, include its include paths in the build configuration.
            for path in lib.include_paths {
                build.include(path);
            }
        }
        Err(_) => {
            // If pkg-config fails, fall back to a default include path.
            eprintln!("⚠️ pkg-config failed, falling back to default include path");
            build.include("/usr/include");
        }
    }

    // Configure the build to compile the drmhook.c file and generate a shared library.
    // The compiler command is customized with flags for a position-independent code (-fPIC)
    // and links against libdrm and libdl.
    build
        .get_compiler()
        .to_command()
        .args([
            "-shared",
            "-fPIC",
            "lib/drmhook.c",
            "-o",
            &out_lib,
            "-ldrm",
            "-ldl",
        ])
        .args(&objects)
        .status()
        .expect("Failed to execute compiler command");

    // Construct the path to the compiled library in the output directory.
    let lib_path = Path::new(&out_dir).join(lib_name);

    // Construct the path where the library will be copied for easier access.
    let final_copy_path = format!("target/{}/{}", profile, lib_name);

    // Create the target/profile directory if it doesn't exist.
    fs::create_dir_all(format!("target/{}", profile)).expect("Failed to create target directory");

    // Copy the compiled library to the target/profile directory for debugging or manual access.
    fs::copy(&lib_path, &final_copy_path).expect("Failed to copy library to target directory");

    // Instruct Cargo to link against the compiled library by specifying the search path.
    println!("cargo:rustc-link-search=native={}", out_dir);

    // Instruct Cargo to link the drmhook library as a dynamic library.
    println!("cargo:rustc-link-lib=dylib=drmhook");

    // Inform Cargo para reexecutar o script de build se o arquivo drmhook.c for alterado.
    println!("cargo:rerun-if-changed=lib/drmhook.c");
}
