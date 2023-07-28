use super::*;
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
#[derive(Clone, Debug)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub enum Command {
    MoveByCommand(f64, f64),
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
                player.dx = *dx;
                player.dy = *dy;
            }
        }
    }
}
