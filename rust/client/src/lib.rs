use godot::prelude::*;

mod armor;
mod map;
mod players;
mod weapon;

struct Client;

#[gdextension]
unsafe impl ExtensionLibrary for Client {}
