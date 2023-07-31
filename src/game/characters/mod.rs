use super::{commands::AbilityId, *};

#[derive(Clone, Debug)]
pub struct Minkle {
    pub drone_id: GameObjectId,
    drone_target: Option<(f64, f64)>,
}

#[derive(Clone, Debug)]
pub enum Character {
    Minkle,
}

impl Minkle {
    pub fn new(game: &mut Game, minkle_id: GameObjectId) -> GameObjectId {
        let (x, y) = {
            let pos = game
                .positions
                .get(&minkle_id)
                .expect("Minkle created without object!");
            (pos.x, pos.y)
        };
        let drone_id = game.create_game_object(x + 10.0, y);
        GravityAffected::new(game, minkle_id);
        game.characters.insert(minkle_id, Character::Minkle);
        game.minkles.insert(
            minkle_id,
            Minkle {
                drone_id,
                drone_target: None,
            },
        );
        minkle_id
    }
    pub fn step(game: &mut Game) {
        for (id, minkle) in game.minkles.iter_mut() {
            if let Some((tx, ty)) = minkle.drone_target {
                if let Some(drone_pos) = game.positions.get_mut(&minkle.drone_id) {
                    let mut xtp = tx - drone_pos.x;
                    let mut ytp = ty - drone_pos.y;
                    let mag_sq = xtp * xtp + ytp * ytp;
                    let mag = mag_sq.sqrt();
                    if mag > DRONE_SPEED {
                        xtp *= DRONE_SPEED / mag;
                        ytp *= DRONE_SPEED / mag;
                    } else {
                        minkle.drone_target = None;
                    }
                    drone_pos.x += xtp;
                    drone_pos.y += ytp;
                }
            }
        }
    }
}

const DRONE_SPEED: f64 = 7.5;
impl Character {
    pub fn apply_ability_command(
        game: &mut Game,
        id: GameObjectId,
        _ability_id: AbilityId,
        tx: f64,
        ty: f64,
    ) {
        match game.characters.get_mut(&id) {
            Some(Character::Minkle) => {
                game.minkles.get_mut(&id).unwrap().drone_target = Some((tx, ty));
            }
            None => {}
        }
    }
}
