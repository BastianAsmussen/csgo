use godot::engine::PhysicsRayQueryParameters3D;
use godot::prelude::*;

use crate::players::player::Player;

#[derive(Debug, GodotConvert, PartialEq, Var, Export)]
#[godot(via = GString)]
pub enum Phase {
    Search,
    Attack,
    Chase,
    Retreat,
}

#[derive(Debug, GodotClass)]
#[class(init, base = Node3D)]
pub struct Bot {
    #[export]
    #[init(default = Phase::Search)]
    phase: Phase,
    #[export]
    #[init(default = None)]
    target: Option<Gd<Player>>,

    #[export]
    #[init(default = 10.0)]
    sight_range: f32,

    #[init(default = None)]
    player: Option<Gd<Player>>,

    base: Base<Node3D>,
}

#[godot_api]
impl Bot {
    #[func]
    pub fn target(&self) -> Option<Gd<Player>> {
        self.target.clone()
    }

    #[func]
    pub fn player(&self) -> Option<Gd<Player>> {
        self.player.clone()
    }
}

#[godot_api]
impl INode3D for Bot {
    fn ready(&mut self) {
        let parent = match self.base().get_parent() {
            Some(parent) => parent.cast::<Player>(),
            None => {
                godot_error!("Bot must be a child of a node!");

                return;
            }
        };

        self.player = Some(parent);
    }

    fn process(&mut self, delta: f64) {
        let Some(mut player) = self.player() else {
            return;
        };

        // 1. Move around randomly and attempt to spot an enemy player.
        // 2. If an enemy is spotted, shoot at them.
        // 3. If the enemy moves out of sight, chase them to last known location.
        // 4. If not found, return to step 1.

        match self.phase {
            Phase::Search => {
                // Move around randomly.
                let mut direction = player.get_velocity();
                direction.x = (rand::random::<f32>() - 0.5) * 2.0;
                direction.z = (rand::random::<f32>() - 0.5) * 2.0;

                let direction = direction * player.bind().run_speed();

                player.set_velocity(direction);

                // Cast a ray to check if an enemy is in LoS.
                let Some(mut world) = player.get_world_3d() else {
                    return;
                };
                let Some(mut space_state) = world.get_direct_space_state() else {
                    return;
                };

                let cam = self.base().get_node_as::<Camera3D>("../Camera");
                let center = match cam.get_viewport() {
                    Some(view_port) => view_port.get_visible_rect().center(),
                    None => return,
                };

                let origin = cam.project_ray_origin(center);
                let end = origin + cam.project_ray_normal(center) * self.sight_range;

                let Some(mut query) = PhysicsRayQueryParameters3D::create(origin, end) else {
                    return;
                };
                query.set_collide_with_areas(true);

                let result = space_state.intersect_ray(query);

                // Get the player that wes hit.
                let Some(collider) = result.get("collider") else {
                    return;
                };

                let Some(target) = collider.try_to::<Gd<Player>>().ok() else {
                    return;
                };

                self.phase = Phase::Attack;
                self.target = Some(target);
            }
            Phase::Attack => {
                let Some(target) = self.target() else {
                    self.phase = Phase::Search;

                    return;
                };

                // Look at the player.
                let target_position = target.get_global_transform().origin;
                let distance = target_position - self.base().get_global_transform().origin;
                let _direction = distance * delta as f32;

                self.base_mut().look_at(target_position);

                // If we're looking at the player, shoot.
                let mut weapon = player.bind().weapon();
                if let Some(w) = weapon.as_mut() {
                    if !w.bind().has_ammo() {
                        self.phase = Phase::Retreat;
                    }

                    w.bind_mut().fire();
                };

                // If the target is dead, return to searching.
                if target.bind().is_dead() {
                    self.phase = Phase::Search;
                    self.target = None;
                }
            }
            Phase::Chase => {
                let Some(target) = self.target.as_ref() else {
                    self.phase = Phase::Search;

                    return;
                };

                // Chase the enemy to last known location.
                let target_position = target.get_global_transform().origin;
                let distance = target_position - self.base().get_global_transform().origin;
                let direction = distance.normalized() * player.bind().run_speed();

                player.set_velocity(direction);

                // If we're close enough, attack.
                if distance.length() < self.sight_range {
                    self.phase = Phase::Attack;
                }
            }
            Phase::Retreat => {
                // Retreat from the target.
                let Some(target) = self.target.as_ref() else {
                    self.phase = Phase::Search;

                    return;
                };

                let target_position = target.get_global_transform().origin;
                let distance = target_position - self.base().get_global_transform().origin;
                let direction = distance.normalized() * player.bind().run_speed();

                player.set_velocity(-direction);
            }
        }
    }
}
