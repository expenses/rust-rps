fn bind_shader(parent_dir: &str, shader: &str) {
    println!("cargo:rustc-link-search={}", parent_dir);
    println!("cargo:rustc-link-lib=static={}", shader);
    println!("cargo:rerun-if-changed={}/lib{}.a", parent_dir, shader);
}

fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let source_file = "callback_runtime.cpp";
    let object_file = out_path.join("callback_runtime.o");
    let archive_file = out_path.join("libcallback_runtime.a");

    let bindings = bindgen::Builder::default()
        .header(source_file)
        .opaque_type("std::.*")
        .allowlist_function("add_callback_runtime")
        .allowlist_function("rps[^:]*")
        .allowlist_type("rps[^:]*")
        .allowlist_type("Rps.*")
        .allowlist_type("PFN.*")
        .allowlist_type("rps::RenderGraphUpdateContext")
        .allowlist_type("Callbacks")
        .opaque_type("rps::Arena")
        .opaque_type(".*Pool.*")
        // This inherits from ParamDecl and is thus pretty broken.
        .opaque_type("rps::NodeParamDecl")
        .rustified_enum("Rps.*")
        .enable_cxx_namespaces()
        .clang_args([
            "-I",
            "RenderPipelineShaders/include",
            "-I",
            "RenderPipelineShaders/src",
            "-stdlib=libc++",
            "-x",
            "c++",
        ])
        .impl_debug(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-search=RenderPipelineShaders/build/src");
    println!("cargo:rustc-link-search={}", out_path.display());

    println!("cargo:rustc-link-lib=static=callback_runtime");
    println!("cargo:rustc-link-lib=rps_runtime");
    println!("cargo:rustc-link-lib=rps_core");
    println!("cargo:rustc-link-lib=stdc++");

    bind_shader("pipeline_shaders", "upscale");
    bind_shader("pipeline_shaders", "rps_multithreading");

    println!("cargo:rerun-if-changed={}", source_file);

    if !std::process::Command::new("clang")
        .arg("-c")
        .arg("-o")
        .arg(&object_file)
        .arg(source_file)
        .arg("-IRenderPipelineShaders/src")
        .arg("-IRenderPipelineShaders/include")
        .output()
        .expect("could not spawn `clang`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not compile object file");
    }

    if !std::process::Command::new("ar")
        .arg("rcs")
        .arg(&archive_file)
        .arg(&object_file)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not emit library file");
    }
}
