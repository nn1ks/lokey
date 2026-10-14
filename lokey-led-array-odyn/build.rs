fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/odyn_led_array.capnp")
        .file("schema/odyn_led_array_capability_id.capnp")
        .file("schema/odyn_led_array_message_id.capnp")
        .run()
        .expect("schema compiler command");
}
