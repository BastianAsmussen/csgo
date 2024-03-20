use godot::engine::{
    CollisionShape3D, IRigidBody3D, PhysicsRayQueryParameters3D, RigidBody3D, SphereShape3D, Timer,
};
use godot::prelude::*;

use crate::player::Player;

#[derive(Debug, PartialEq, GodotConvert, Var, Export)]
#[godot(via = GString)]
enum Headshot {
    Kill,
    DoubleDamage,
}

#[derive(Debug, GodotClass)]
#[class(base = RigidBody3D)]
pub struct Weapon {
    #[export]
    max_damage: f64,
    #[export]
    min_damage: f64,

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
    on_headshot: Headshot,

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
        let end = origin + cam.project_ray_normal(mouse_pos) * self.falloff_end as f32;

        let mut query = PhysicsRayQueryParameters3D::create(origin, end)?;
        query.set_collide_with_areas(true);

        let result = space_state.intersect_ray(query);

        // Get the player that wes hit.
        let collider = result.get("collider")?;
        let position = result.get("position")?.try_to::<Vector3>().ok()?;

        let mut player = collider.try_to::<Gd<Player>>().ok()?;
        let distance = position.distance_to(origin);

        // If the player's head was hit, deal max damage.
        // Determine this by checking if the hit position is within the head collider.
        let head = player.get_node_as::<CollisionShape3D>("HeadCollider");
        godot_print!("Head: {head:?}");

        let is_headshot = head
            .cast::<SphereShape3D>()
            .map_or(false, |s| s.is_point_inside(position));
        godot_print!("Distance: {distance}, Headshot? {is_headshot}");

        let mut damage = self.calculate_damage(distance as f64);
        if is_headshot {
            damage = match self.on_headshot {
                Headshot::Kill => player.bind().max_health(),
                Headshot::DoubleDamage => damage * 2.0,
            };
        }

        player.bind_mut().damage(damage);

        Some(player)
    }

    #[func]
    pub fn calculate_damage(&self, distance: f64) -> f64 {
        if distance <= self.falloff_start {
            return self.max_damage;
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
    fn init(base: Base<RigidBody3D>) -> Self {
        Self {
            max_damage: 0.0,
            min_damage: 0.0,

            falloff_start: 0.0,
            falloff_end: 0.0,

            fire_rate: None,

            max_ammo: 0,
            current_ammo: 0,
            on_headshot: Headshot::Kill,

            reload_time: None,

            base,
        }
    }
}
