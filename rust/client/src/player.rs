use godot::engine::{CharacterBody3D, ICharacterBody3D, InputEvent, InputEventMouseMotion};
use godot::prelude::*;

use crate::weapon::Weapon;

use std::io::{Read, Write};
use std::net::TcpStream;

use serde::ser::SerializeStruct;
use serde::Serialize;

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, GodotConvert, Var, Export)]
#[godot(via = GString)]
enum Owner {
    Client,
    Server,
}

#[derive(Debug, GodotClass)]
#[class(init, base = CharacterBody3D)]
pub struct Player {
    #[export]
    name: GString,

    #[export]
    #[init(default = Owner::Server)]
    owner: Owner,

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

    socket: Option<TcpStream>,

    base: Base<CharacterBody3D>,
}

impl Serialize for Player {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Player", 4)?;
        state.serialize_field("name", &self.name.to_string())?;
        state.serialize_field("health", &self.health)?;
        state.serialize_field("max_health", &self.max_health)?;
        state.serialize_field(
            "position",
            &self.base().get_global_transform().origin.to_string(),
        )?;

        state.end()
    }
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
            godot_print!("I'm dead!");

            return;
        }

        let health_percentage = (self.health() / self.max_health()) * 100.0;
        godot_print!("Ouch! I was hit for {damage:.2} damage! ({health_percentage:.2}%)");
    }
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn ready(&mut self) {
        if self.owner == Owner::Server {
            return;
        }

        let id = self.base().instance_id();
        std::thread::spawn(|| {
            let mut socket = TcpStream::connect("127.0.0.1:7512").unwrap();

            loop {
                let player = Player::from_instance_id(id);
                let mut buffer = [0; 1024];
                let data = match socket.read(&mut buffer) {
                    Ok(0) => {
                        // The server has disconnected.
                        return;
                    }
                    Ok(n) => {
                        let data = &buffer[..n];
                        let data = String::from_utf8_lossy(data);

                        data.to_string()
                    }
                    Err(e) => {
                        godot_print!("Error reading from server: {e}");

                        return;
                    }
                };

                if data == "GetState" {
                    godot_print!("Sending state to server...");

                    let state = match serde_json::to_string(&player) {
                        Ok(state) => state,
                        Err(e) => {
                            godot_print!("Error serializing state: {e}");

                            return;
                        }
                    };

                    godot_print!("State: {state}");

                    match socket.write(state.as_bytes()) {
                        Ok(_) => {}
                        Err(e) => {
                            godot_print!("Error writing to server: {e}");

                            return;
                        }
                    }

                    continue;
                }
            }
        });
    }

    fn physics_process(&mut self, delta: f64) {
        // Apply gravity.
        let mut velocity = self.base().get_velocity();
        velocity.y -= (self.gravity * delta) as f32;

        self.base_mut().set_velocity(velocity);

        // Handle input and move the player.
        if self.owner == Owner::Client {
            let direction = self.handle_input();
            self.base_mut().set_velocity(direction);
        }

        self.base_mut().move_and_slide();
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if self.owner == Owner::Server {
            return;
        }

        if let Ok(event) = event.try_cast::<InputEventMouseMotion>() {
            /*
             $Camera.rotate_y(deg2rad(-event.relative.x*mouse_sens))
             var changev=-event.relative.y*mouse_sens
             if camera_anglev+changev>-50 and camera_anglev+changev<50:
                camera_anglev+=changev
                $Camera.rotate_x(deg2rad(changev))
            */
            let y_change = (-event.get_relative().x * self.mouse_sensitivty).clamp(-90.0, 90.0);
            let x_change = -event.get_relative().y * self.mouse_sensitivty;

            let rotation = self.base().get_rotation_degrees();
            let rotation = Vector3::new(rotation.x + x_change, rotation.y + y_change, rotation.z);

            self.base_mut().set_rotation_degrees(rotation);
        }
    }
}
