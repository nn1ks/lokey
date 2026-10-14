fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/odyn_keyboard.capnp")
        .file("schema/odyn_keyboard_action_id.capnp")
        .file("schema/odyn_keyboard_message_id.capnp")
        .default_parent_module(["protocol".to_owned()].into())
        .run()
        .expect("schema compiler command");
}
