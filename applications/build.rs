fn main() {
    #[cfg(feature = "cuda")]
    {
        use std::{env, path::PathBuf, process::Command};

        #[allow(deprecated)]
        use bindgen::CargoCallbacks;
        use regex::Regex;

        println!("cargo:rerun-if-changed=generate_grid/cuda");

        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

        let kernels_dir = manifest_dir.join("generate_grid/cuda/kernels");
        let kernels = [("hex.cu", "hex.ptx"), ("square.cu", "square.ptx")];
        for (src, out) in kernels {
            let cuda_src = kernels_dir.join(src);
            let cuda_out = out_dir.join(out);
            let nvcc_status = Command::new("nvcc")
                .arg("-ptx")
                .arg("-o")
                .arg(&cuda_out)
                .arg(cuda_src)
                .status()
                .unwrap();
            assert!(
                nvcc_status.success(),
                "Failed to compile CUDA source to PTX."
            );
        }

        let bindings = bindgen::Builder::default()
            .header(
                manifest_dir
                    .join("generate_grid/cuda/includes/wrapper.h")
                    .to_string_lossy(),
            )
            .blocklist_type(".*_t")
            .blocklist_type("__u_.*")
            .blocklist_item("__glibc_c99_flexarr_available")
            // Tell cargo to invalidate the built crate whenever any of the included header files changed.
            .parse_callbacks(Box::new(CargoCallbacks::new()))
            // we use "no_copy" and "no_debug" here because we don't know if we can safely generate them for our structs in C code (they may contain raw pointers)
            .no_copy("*")
            .no_debug("*")
            // Finish the builder and generate the bindings.
            .generate()
            .expect("Unable to generate bindings");

        // we need to make modifications to the generated code
        let generated_bindings = bindings.to_string();

        // Regex to find raw pointers to float and replace them with CudaSlice<f32>
        // You can copy this regex to add/modify other types of pointers, for example "*mut i32"
        let pointer_regex = Regex::new(r"\*mut f32").unwrap();
        let modified_bindings = pointer_regex.replace_all(&generated_bindings, "CudaSlice<f32>");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        std::fs::write(out_path.join("bindings.rs"), modified_bindings.as_bytes())
            .expect("Failed to write bindings");
    }
}
