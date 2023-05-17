use std::collections::HashMap;

#[derive(Debug)]
struct BindgenCallbacks {
    item_replacements: HashMap<&'static str, &'static str>,
    field_name_replacements: HashMap<(&'static str, &'static str), &'static str>,
}

impl BindgenCallbacks {
    fn new() -> Self {
        let mut item_replacements = HashMap::new();
        item_replacements.insert(
            "ResourceDescPacked__bindgen_ty_1",
            "ResourceBufferImageDescPacked",
        );
        item_replacements.insert(
            "ResourceDescPacked__bindgen_ty_1__bindgen_ty_1",
            "ResourceImageDescPacked",
        );
        item_replacements.insert(
            "ResourceDescPacked__bindgen_ty_1__bindgen_ty_2",
            "ResourceBufferDescPacked",
        );
        item_replacements.insert(
            "ResourceDescPacked__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1",
            "ResourceImageDepthArrayLayersDescPacked",
        );

        let mut field_name_replacements = HashMap::new();

        field_name_replacements.insert(("ResourceDescPacked", "__bindgen_anon_1"), "buffer_image");
        field_name_replacements.insert(
            ("ResourceImageDescPacked", "__bindgen_anon_1"),
            "depth_array_layers",
        );

        Self {
            item_replacements,
            field_name_replacements,
        }
    }
}

impl bindgen::callbacks::ParseCallbacks for BindgenCallbacks {
    fn process_comment(&self, comment: &str) -> Option<String> {
        if comment.contains("```") {
            let mut new = String::new();
            let mut first = true;
            let mut start_comment = false;

            for section in comment.split("```") {
                if !first {
                    if start_comment {
                        new.push_str("```text")
                    } else {
                        new.push_str("```");
                    }
                }
                start_comment = !start_comment;
                first = false;
                new.push_str(section);
            }

            Some(new)
        } else {
            None
        }
    }

    fn item_name(&self, original_item_name: &str) -> Option<String> {
        if let Some(&replacement) = self.item_replacements.get(original_item_name) {
            Some(String::from(replacement))
        } else {
            None
        }
    }

    fn process_field_name(&self, parent_name: &str, name: &str) -> Option<String> {
        if let Some(&replacement) = self.field_name_replacements.get(&(parent_name, name)) {
            Some(String::from(replacement))
        } else {
            None
        }
    }
}

fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let source_file = "callback_runtime.cpp";
    let object_file = out_path.join("callback_runtime.o");
    let archive_file = out_path.join("libcallback_runtime.a");

    let bindings = bindgen::Builder::default()
        .header(source_file)
        .module_raw_line("root", "pub use render_pipeline_shaders_sys::*;")
        // Make std opaque and blocklisted.
        .opaque_type("std::.*")
        .blocklist_type("std::.*")
        // Add custom types
        .allowlist_type("Callbacks")
        .allowlist_function("add_callback_runtime")
        // Add rps c++ types
        .allowlist_function("rps::.*")
        .allowlist_type("rps::.*")
        // Exceptions to the above
        .opaque_type(".*FreeListPool.*")
        .opaque_type("rps::NodeParamDecl")
        .blocklist_type("rps::.*iterator")
        // Make rps c types opaque and blocklist c functions
        .opaque_type("Rps.*")
        .blocklist_function("rps.*")
        // Blocklist all types that would become typedefs.
        // Some C++ types inherit from the C types so we can't blocklist
        // those.
        .blocklist_type("RpsRuntimeResource")
        .blocklist_type("RpsSubprogram")
        .blocklist_type("RpsRuntimeHeap")
        .blocklist_type("RpsRuntimeCommandBuffer")
        .blocklist_type("RpsParamAttrList")
        .blocklist_type("RpsScheduleFlags")
        .blocklist_type("RpsNodeDeclFlags")
        .blocklist_type("RpsParameterFlags")
        .blocklist_type("RpsNodeFlags")
        .blocklist_type("RpsQueueFlags")
        .blocklist_type("RpsRuntimeRenderPassFlags")
        .blocklist_type("RpsBool")
        .blocklist_type("RpsDevice")
        .blocklist_type("RpsNodeDeclId")
        .blocklist_type("RpsParamId")
        .blocklist_type("RpsNodeId")
        .blocklist_type("RpsVariable")
        .blocklist_type("RpsConstant")
        .blocklist_type("RpsSubgraphFlags")
        .blocklist_type("RpslEntryCallFlags")
        .blocklist_type("RpsResourceId")
        .blocklist_type("RpsResourceFlags")
        .blocklist_type("RpsRuntimeRenderPassFlags")
        .blocklist_type("RpsImageAspectUsageFlags")
        .blocklist_type("RpsAccessFlags")
        .blocklist_type("RpsShaderStageFlags")
        .blocklist_type("RpsResourceViewFlags")
        .blocklist_type("RpsRecordCommandFlags")
        .blocklist_type("RpsRenderGraphDiagnosticInfoFlags")
        .clang_args([
            "-I",
            "RenderPipelineShaders/include",
            "-I",
            "RenderPipelineShaders/src",
            "-stdlib=libc++",
            "-x",
            "c++",
        ])
        .enable_cxx_namespaces()
        .parse_callbacks(Box::new(BindgenCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-search={}", out_path.display());
    println!("cargo:rustc-link-lib=callback_runtime");

    println!("cargo:rerun-if-changed={}", source_file);
    println!("cargo:rerun-if-changed=RenderPipelineShaders");

    if !std::process::Command::new("clang")
        .arg("-c")
        .arg("-o")
        .arg(&object_file)
        .arg(source_file)
        .arg("-g")
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
