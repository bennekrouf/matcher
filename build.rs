use std::env;
use std::path::PathBuf;

fn main() {
    tonic_build::compile_protos("proto/health.proto").unwrap();
    // Get the OUT_DIR environment variable at runtime
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Construct the path to the descriptor set file (optional)
    let descriptor_path = out_dir.join("matcher_descriptor.bin");

    // Configure and compile the proto files
    tonic_build::configure()
        .file_descriptor_set_path(descriptor_path) // Optional: for advanced usage
        .compile_protos(&["proto/matcher.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile proto files: {}", e));
}
