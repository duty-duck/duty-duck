fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["../protos/http-monitor-executor.proto"], &["../protos"])
        .expect("Failed to compile protos");
}
