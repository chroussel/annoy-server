extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .src_prefix("src")
        .file("src/service.capnp")
        .run()
        .expect("compiling schema");
}
