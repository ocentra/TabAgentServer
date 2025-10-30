fn main() {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../protos/database.proto", "../protos/ml_inference.proto"],
            &["../protos"],
        )
        .unwrap();
}
