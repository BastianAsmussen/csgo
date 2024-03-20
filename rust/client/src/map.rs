use godot::prelude::*;

use crate::player::Player;

#[derive(Debug, GodotClass)]
#[class(base = Node3D)]
pub struct Map {
    #[export]
    players: Array<Gd<Player>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Map {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            players: Array::new(),

            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        for i in 0..self.players.len() {
            let player = self.players.get(i);
            let player = player.bind();
            if player.is_dead() {
                godot_print!("{} is dead!", player.name())
            }
        }
    }
}
