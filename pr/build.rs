fn main() {
    tonic_build::compile_protos("../proto/incoming.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    tonic_build::compile_protos("../proto/storage.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
