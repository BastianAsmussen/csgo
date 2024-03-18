use godot::engine::{CharacterBody3D, ICharacterBody3D};
use godot::prelude::*;

use crate::weapon::Weapon;

#[derive(GodotClass)]
#[class(base = CharacterBody3D)]
pub struct Player {
    #[export]
    gravity: f32,

    #[export]
    run_speed: f32,
    #[export]
    jump_force: f32,

    #[export]
    weapon: Option<Gd<Weapon>>,

    base: Base<CharacterBody3D>,
}

#[godot_api]
impl Player {
    const GRAVITY: f32 = 9.8;

    const RUN_SPEED: f32 = 10.0;
    const JUMP_FORCE: f32 = 5.0;

    #[func]
    fn handle_input(&mut self) -> Vector3 {
        let mut velocity = self.base().get_velocity();

        velocity.z = 0.0;
        velocity.x = 0.0;

        let input = Input::singleton();
        let is_action_pressed = |action: &str| input.is_action_pressed(action.into());

        if is_action_pressed("move_forward") {
            velocity.z -= self.run_speed;
        }
        if is_action_pressed("move_backward") {
            velocity.z += self.run_speed;
        }
        if is_action_pressed("move_left") {
            velocity.x -= self.run_speed;
        }
        if is_action_pressed("move_right") {
            velocity.x += self.run_speed;
        }
        if is_action_pressed("jump") && self.base().is_on_floor() {
            velocity.y = self.jump_force;
        }

        if is_action_pressed("fire_weapon") {
            if let Some(weapon) = &mut self.weapon {
                weapon.bind_mut().fire();
            }
        }

        velocity
    }
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn init(base: Base<CharacterBody3D>) -> Self {
        Self {
            gravity: Self::GRAVITY,

            run_speed: Self::RUN_SPEED,
            jump_force: Self::JUMP_FORCE,

            weapon: None,

            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // Apply gravity.
        let mut velocity = self.base().get_velocity();
        velocity.y -= self.gravity * delta as f32;

        self.base_mut().set_velocity(velocity);

        // Handle input and move the player.
        let direction = self.handle_input();
        self.base_mut().set_velocity(direction);

        self.base_mut().move_and_slide();
    }
}
