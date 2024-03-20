use godot::prelude::*;

mod map;
mod player;
mod weapon;

struct Client;

#[gdextension]
unsafe impl ExtensionLibrary for Client {}
