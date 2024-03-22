use godot::engine::{CharacterBody3D, ICharacterBody3D, InputEvent, InputEventMouseMotion};
use godot::prelude::*;

use crate::weapon::Weapon;

use crate::players::bot::Bot;

#[derive(Debug, GodotClass)]
#[class(init, base = CharacterBody3D)]
pub struct Player {
    #[export]
    name: GString,

    #[export]
    #[init(default = None)]
    bot: Option<Gd<Bot>>,

    #[export]
    #[init(default = 9.8)]
    gravity: f64,

    #[export]
    #[init(default = 10.0)]
    run_speed: f32,
    #[export]
    #[init(default = 5.0)]
    jump_force: f32,

    #[export]
    #[init(default = 3.0)]
    mouse_sensitivty: f32,

    #[export]
    #[init(default = 100.0)]
    max_health: f64,
    #[export]
    #[init(default = 100.0)]
    health: f64,

    #[export]
    weapon: Option<Gd<Weapon>>,

    base: Base<CharacterBody3D>,
}

#[godot_api]
impl Player {
    #[func]
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    #[func]
    pub fn max_health(&self) -> f64 {
        self.max_health
    }

    #[func]
    pub fn health(&self) -> f64 {
        self.health
    }

    #[func]
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    #[func]
    pub fn weapon(&self) -> Option<Gd<Weapon>> {
        self.weapon.clone()
    }

    #[func]
    pub fn run_speed(&self) -> f32 {
        self.run_speed
    }

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
            self.weapon.as_mut().map(|w| w.bind_mut().fire());
        }

        velocity
    }

    #[func]
    pub fn damage(&mut self, damage: f64) {
        self.health -= damage.min(self.health);

        if self.is_dead() {
            // Show death screen.
            if let Some(mut tree) = self.base().get_tree() {
                godot_print!("Pausing...");

                tree.set_pause(true);
            }

            godot_print!("I'm dead!");

            return;
        }

        let health_percentage = (self.health() / self.max_health()) * 100.0;
        godot_print!("Ouch! I was hit for {damage:.2} damage! ({health_percentage:.2}%)");
    }
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn physics_process(&mut self, delta: f64) {
        // Apply gravity.
        let mut velocity = self.base().get_velocity();
        velocity.y -= (self.gravity * delta) as f32;

        self.base_mut().set_velocity(velocity);

        // Handle input and move the player.
        if self.bot.is_none() {
            let direction = self.handle_input();
            self.base_mut().set_velocity(direction);
        }

        self.base_mut().move_and_slide();
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if self.bot.is_some() {
            return;
        }

        if let Ok(event) = event.try_cast::<InputEventMouseMotion>() {
            let rotation = self.base().get_rotation_degrees();
            let mut rotation = rotation
                + Vector3::new(
                    -event.get_relative().y * self.mouse_sensitivty,
                    -event.get_relative().x * self.mouse_sensitivty,
                    0.0,
                );

            // Clamp the rotation to prevent the camera from looking too far up or down.
            rotation.x = rotation.x.min(50.0).max(-50.0);

            self.base_mut().set_rotation_degrees(rotation);
        }
    }
}
