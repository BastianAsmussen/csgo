use godot::prelude::*;

mod player;
mod weapon;

struct Client;

#[gdextension]
unsafe impl ExtensionLibrary for Client {}
