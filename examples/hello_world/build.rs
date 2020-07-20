fn main() {
    prost_build::compile_protos(&["proto/helloworld.proto"], &["proto/"]).unwrap();
}
