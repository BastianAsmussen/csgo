use godot::engine::{
    CollisionShape3D, IRigidBody3D, PhysicsRayQueryParameters3D, RigidBody3D, SphereShape3D, Timer,
};
use godot::prelude::*;

use crate::player::Player;

#[derive(Debug, PartialEq, GodotConvert, Var, Export)]
#[godot(via = GString)]
enum HeadshotAction {
    Kill,
    DoubleDamage,
}

#[derive(Debug, GodotClass)]
#[class(init, base = RigidBody3D)]
pub struct Weapon {
    #[export]
    max_damage: f64,
    #[export]
    min_damage: f64,

    #[export]
    max_range: f64,
    #[export]
    falloff_start: f64,
    #[export]
    falloff_end: f64,

    #[export]
    fire_rate: Option<Gd<Timer>>,

    #[export]
    max_ammo: u32,
    #[export]
    current_ammo: u32,

    #[export]
    #[init(default = HeadshotAction::Kill)]
    on_headshot: HeadshotAction,

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

        let mut world = self.base().get_world_3d()?;
        let mut space_state = world.get_direct_space_state()?;

        let cam = self.base().get_node_as::<Camera3D>("../Camera");
        let mouse_pos = cam.get_viewport()?.get_mouse_position();

        let origin = cam.project_ray_origin(mouse_pos);
        let end = origin + cam.project_ray_normal(mouse_pos) * self.max_range as f32;

        let mut query = PhysicsRayQueryParameters3D::create(origin, end)?;
        query.set_collide_with_areas(true);

        let result = space_state.intersect_ray(query);

        // Get the player that wes hit.
        let collider = result.get("collider")?;
        let position = result.get("position")?.try_to::<Vector3>().ok()?;

        let mut player = collider.try_to::<Gd<Player>>().ok()?;
        let distance = position.distance_to(origin);

        // If the player's head was hit, deal headshot damage.
        // Determine this by checking if the hit position is within the head collider.
        let is_headshot = {
            let collider = player.get_node_as::<CollisionShape3D>("HeadCollider");
            let shape = collider.get_shape()?.cast::<SphereShape3D>();

            // Get the distance from the center of the head collider to the hit position.
            let transform = collider.get_global_transform();
            let distance = transform.origin.distance_to(position) - shape.get_radius();

            // If it's within 0.1 units, we consider it a headshot.
            distance <= 0.1
        };

        let mut damage = self.calculate_damage(distance as f64);
        if is_headshot {
            damage = match self.on_headshot {
                HeadshotAction::Kill => player.bind().max_health(),
                HeadshotAction::DoubleDamage => damage * 2.0,
            };
        }

        player.bind_mut().damage(damage);

        if let Some(timer) = &mut self.fire_rate {
            timer.start();
        };

        if !self.has_ammo() {
            self.reload();
        }

        Some(player)
    }

    #[func]
    pub fn calculate_damage(&self, distance: f64) -> f64 {
        // Control the damage falloff based on the distance.
        if distance <= self.falloff_start {
            return self.max_damage;
        } else if distance >= self.falloff_end {
            return self.min_damage;
        }

        let n = (distance - self.falloff_start) / (self.falloff_end - self.falloff_start);

        n * self.min_damage + (1.0 - n) * self.max_damage
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
    fn ready(&mut self) {
        // Do some validation of the weapon's properties.
        let mut errors = Vec::new();

        if self.max_damage < 0.0 {
            errors.push("max_damage must be greater than or equal to 0.0!");
        }

        if self.min_damage > self.max_damage {
            errors.push("max_damage must be greater than or equal to min_damage!");
        }

        if self.falloff_start < 0.0 {
            errors.push("falloff_start must be greater than or equal to 0.0!");
        }

        if self.falloff_end < 0.0 {
            errors.push("falloff_end must be greater than or equal to 0.0!");
        }

        if self.falloff_start > self.falloff_end {
            errors.push("falloff_start must be less than or equal to falloff_end!");
        }

        if self.max_range < self.falloff_end {
            errors.push("max_range must be greater than or equal to falloff_end!");
        }

        if self.current_ammo > self.max_ammo {
            errors.push("current_ammo must be less than or equal to max_ammo!");
        }

        if errors.is_empty() {
            return;
        }

        godot_print!("Weapon validation failed!");
        for error in errors {
            godot_error!("{error}");
        }
    }
}
