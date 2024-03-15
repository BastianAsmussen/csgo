use godot::prelude::*;

mod player;

struct Client;

#[gdextension]
unsafe impl ExtensionLibrary for Client {}
