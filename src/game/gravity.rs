use super::*;

#[derive(Clone, Debug)]
pub struct GravityAffected {
    current_velocity: f64,
}

pub const GRAVITY_ACCELERATION: f64 = 2.0;
pub const FLOOR_HEIGHT: f64 = 20.0;
impl GravityAffected {
    pub fn step(game: &mut Game) {
        let mut to_delete = Vec::new();
        for (id, gravity_affected) in game.gravity_affected.iter_mut() {
            if let Some(pos) = game.positions.get_mut(id) {
                if pos.y <= FLOOR_HEIGHT {
                    gravity_affected.current_velocity = 0.0;
                } else {
                    pos.y = FLOOR_HEIGHT.max(pos.y - gravity_affected.current_velocity);
                    gravity_affected.current_velocity += GRAVITY_ACCELERATION;
                }
            } else {
                to_delete.push(*id);
            }
        }
        for id in to_delete {
            game.gravity_affected.remove(&id);
        }
    }
    pub fn new(game: &mut Game, id: GameObjectId) {
        let gravity_affected = GravityAffected {
            current_velocity: 0.0,
        };
        game.gravity_affected.insert(id, gravity_affected);
    }
}
