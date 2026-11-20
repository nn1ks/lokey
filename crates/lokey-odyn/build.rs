fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/odyn_core.capnp")
        .file("schema/odyn_core_device_id.capnp")
        .file("schema/odyn_core_error_code.capnp")
        .file("schema/odyn_core_message_id.capnp")
        .default_parent_module(["protocol".to_owned()].into())
        .run()
        .expect("schema compiler command");
}
