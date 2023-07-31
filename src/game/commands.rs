use crate::PLAYER_JUMP_SPEED;

use super::{*, gravity::FLOOR_HEIGHT};
use alkahest::alkahest;

#[derive(Clone, Debug)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct Handshake {
    pub my_name: String,
}
#[derive(Clone, Debug)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct TimingPacket {
    pub sequence_number: u64,
}
#[derive(Clone, Debug)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct SetInputDelay {
    pub input_delay: u64,
}
#[derive(Clone, Debug, Copy)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct AbilityId(pub u8);
#[derive(Clone, Debug)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub enum Command {
    MoveByCommand(f64, f64),
    AbilityCommand(AbilityId, f64, f64),
}

#[derive(Clone, Debug)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct TimedCommand {
    pub time: u64,
    pub command: Command,
}
impl Command {
    pub fn apply(&self, game: &mut Game, player_id: GameObjectId) {
        match self {
            Command::MoveByCommand(dx, dy) => {
                let player = game.players.get_mut(&player_id).unwrap();
                let pos = game.positions.get(&player_id).unwrap();
                let jump = if pos.y <= FLOOR_HEIGHT {
                    if *dy > 0.0 {
                        PLAYER_JUMP_SPEED
                    }
                    else {
                        0.0
                    }
                }
                else {
                    player.jump
                };
                player.dx = *dx;
                player.dy = *dy; // Note that player can still steer vertically with no jumping
                player.jump = jump;
            }
            Command::AbilityCommand(ability_id, tx, ty) => {
                Character::apply_ability_command(game, player_id, *ability_id, *tx, *ty);
            }
        }
    }
}
