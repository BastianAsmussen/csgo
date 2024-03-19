use godot::engine::{IRigidBody3D, PhysicsRayQueryParameters3D, RigidBody3D, Timer};
use godot::prelude::*;

use crate::player::Player;

#[derive(Debug, GodotClass)]
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
    pub fn fire(&mut self) -> Option<Gd<Player>> {
        if self.is_reloading() || self.is_firing() {
            return None;
        }

        self.take_ammo(1);
        godot_print!("Bang! ({} ammo left)", self.current_ammo);

        if let Some(timer) = &mut self.fire_rate {
            timer.start();
        };

        if !self.has_ammo() {
            self.reload();
        }

        let mut world = self.base().get_world_3d()?;
        let mut space_state = world.get_direct_space_state()?;

        let cam = self.base().get_node_as::<Camera3D>("../Camera");
        let mouse_pos = cam.get_viewport()?.get_mouse_position();

        let origin = cam.project_ray_origin(mouse_pos);
        let end = origin + cam.project_ray_normal(mouse_pos) * self.range as f32;

        let mut query = PhysicsRayQueryParameters3D::create(origin, end)?;
        query.set_collide_with_areas(true);

        let result = space_state.intersect_ray(query);

        // Get the player that wes hit.
        let mut player = result.get("collider")?.try_to::<Gd<Player>>().ok()?;
        let distance = result
            .get("position")?
            .try_to::<Vector3>()
            .ok()?
            .distance_to(origin);

        player
            .bind_mut()
            .damage(self.calculate_damage(distance as f64));

        Some(player)
    }

    #[func]
    pub fn calculate_damage(&self, distance: f64) -> f64 {
        // Calculate damage fall-off.
        let damage = self.damage * (1.0 - (distance / self.range));

        damage
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
