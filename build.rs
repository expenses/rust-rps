fn main() {
    println!("cargo:rustc-link-search=RenderPipelineShaders/build/src");
    println!("cargo:rustc-link-search=.");
    println!("cargo:rustc-link-lib=static=out");
    println!("cargo:rustc-link-lib=static=custom_runtime");
    println!("cargo:rustc-link-lib=rps_runtime");
    println!("cargo:rustc-link-lib=rps_core");
    println!("cargo:rustc-link-lib=stdc++");
}
