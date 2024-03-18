use godot::engine::{IRigidBody3D, PhysicsRayQueryParameters3D, RigidBody3D, Timer};
use godot::prelude::*;

use crate::player::Player;

#[derive(GodotClass)]
#[class(base = RigidBody3D)]
pub struct Weapon {
    #[export]
    damage: f64,
    #[export]
    range: f64,

    #[export]
    fire_rate: Option<Gd<Timer>>,

    #[export]
    max_ammo: u32,
    #[export]
    current_ammo: u32,
    #[export]
    reload_time: Option<Gd<Timer>>,

    base: Base<RigidBody3D>,
}

#[godot_api]
impl Weapon {
    #[func]
    pub fn is_reloading(&self) -> bool {
        self.reload_time
            .as_ref()
            .map_or(false, |timer| !timer.is_stopped())
    }

    #[func]
    pub fn is_firing(&self) -> bool {
        self.fire_rate
            .as_ref()
            .map_or(false, |timer| !timer.is_stopped())
    }

    #[func]
    pub fn fire(&mut self) {
        // -> Vec<Player> {
        if self.is_reloading() || self.is_firing() {
            return; // Vec::new();
        }

        self.take_ammo(1);
        godot_print!("Bang! ({} ammo left)", self.current_ammo);

        if let Some(timer) = &mut self.fire_rate {
            timer.start();
        };

        if !self.has_ammo() {
            self.reload();
        }

        /*
             * var space_state = get_world_3d().direct_space_state
        var cam = $Camera3D
        var mousepos = get_viewport().get_mouse_position()

        var origin = cam.project_ray_origin(mousepos)
        var end = origin + cam.project_ray_normal(mousepos) * RAY_LENGTH
        var query = PhysicsRayQueryParameters3D.create(origin, end)
        query.collide_with_areas = true

        var result = space_state.intersect_ray(query)

             */
        let mut space_state = self
            .base()
            .get_world_3d()
            .unwrap()
            .get_direct_space_state()
            .unwrap();
        let cam = self.base().get_node_as::<Camera3D>("Camera3D");
        let mousepos = self.base().get_viewport().unwrap().get_mouse_position();

        let origin = cam.project_ray_normal(mousepos);
        let end = origin + cam.project_ray_normal(mousepos) * self.range as f32;
        let mut query = PhysicsRayQueryParameters3D::create(origin, end).unwrap();
        query.set_collide_with_areas(true);

        let result = space_state.intersect_ray(query);
    }

    #[func]
    pub fn take_ammo(&mut self, amount: u32) {
        self.current_ammo -= amount.min(self.current_ammo);
    }

    #[func]
    pub fn has_ammo(&self) -> bool {
        self.current_ammo > 0
    }

    #[func]
    pub fn reload(&mut self) {
        if self.is_reloading() {
            return;
        }

        godot_print!("Reloading...");

        if let Some(timer) = &mut self.reload_time {
            timer.start();
        }

        self.current_ammo = self.max_ammo;
    }
}

#[godot_api]
impl IRigidBody3D for Weapon {
    fn init(base: Base<RigidBody3D>) -> Self {
        Self {
            damage: 0.0,
            range: 0.0,

            fire_rate: None,

            max_ammo: 0,
            current_ammo: 0,
            reload_time: None,

            base,
        }
    }
}
