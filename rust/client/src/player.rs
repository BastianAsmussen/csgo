use godot::engine::{CharacterBody3D, ICharacterBody3D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base = CharacterBody3D)]
struct Player {
    #[export]
    foo: i32,

    base: Base<CharacterBody3D>,
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn init(base: Base<CharacterBody3D>) -> Self {
        godot_print!("Player init!");

        Self { foo: 0, base }
    }
}
