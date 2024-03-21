use godot::engine::IRigidBody3D;
use godot::prelude::*;

#[derive(Debug, GodotClass)]
#[class(init, base = RigidBody3D)]
pub struct Armor {
    #[export]
    damage_mitigation: f64,

    #[export]
    max_durability: f64,
    #[export]
    current_durability: f64,

    #[export]
    ricochet_chance: f64,
}

#[godot_api]
impl Armor {
    #[func]
    pub fn take_damage(&mut self, damage: f64) -> f64 {
        if self.current_durability <= 0.0 {
            return damage;
        }

        let mitigated = damage * self.damage_mitigation;
        self.current_durability -= mitigated;

        mitigated
    }

    #[func]
    pub fn is_ricochet(&self) -> bool {
        rand::random::<f64>() < self.ricochet_chance
    }
}

#[godot_api]
impl IRigidBody3D for Armor {
    fn ready(&mut self) {
        // Do some validation of the armor's properties.
        let mut errors = Vec::new();

        if !(0.0..=1.0).contains(&self.damage_mitigation) {
            errors.push("damage_mitigation must be between 0.0 and 1.0!");
        }

        if self.max_durability < 0.0 {
            errors.push("max_durability must be greater than or equal to 0.0!");
        }

        if self.current_durability > self.max_durability {
            errors.push("current_durability must be less than or equal to max_durability!");
        }

        if !(0.0..=1.0).contains(&self.ricochet_chance) {
            errors.push("ricochet_chance must be between 0.0 and 1.0!");
        }

        if errors.is_empty() {
            return;
        }

        godot_print!("Armor validation failed!");
        for error in errors {
            godot_error!("{error}");
        }
    }
}
