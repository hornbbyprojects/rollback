use std::collections::HashMap;
use std::hash::Hash;

pub mod commands;

use sdl2::{
    rect::Rect,
    render::{Canvas, RenderTarget},
};

use self::commands::Command;

#[derive(Hash, Eq, Ord, PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct GameObjectId(u64);

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn get_distance_squared(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
    pub fn is_closer_than(&self, other: &Self, distance: f64) -> bool {
        let distance_sq = self.get_distance_squared(other);
        distance_sq < distance * distance
    }
}

pub struct U64DoNothingHasher {
    value: u64,
    already_written: bool,
}
impl std::hash::Hasher for U64DoNothingHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, bytes: &[u8]) {
        if self.already_written {
            panic!("Wrote to hasher twice!");
        }
        self.value = byteorder::ReadBytesExt::read_u64::<byteorder::BigEndian>(
            &mut std::io::Cursor::new(bytes),
        )
        .expect("Failed to read u64 when hashing");
        self.already_written = true;
    }
}
#[derive(Default, Clone)]
pub struct U64DoNothingBuildHasher {}
impl std::hash::BuildHasher for U64DoNothingBuildHasher {
    type Hasher = U64DoNothingHasher;

    fn build_hasher(&self) -> Self::Hasher {
        U64DoNothingHasher {
            value: 0,
            already_written: false,
        }
    }
}
type IdHashMap<V> = HashMap<GameObjectId, V, U64DoNothingBuildHasher>;

#[derive(Clone)]
pub struct Player {
    pub dx: f64,
    pub dy: f64,
}

impl Player {
    pub fn new(game: &mut Game, x: f64, y: f64) -> GameObjectId {
        let id = game.create_game_object(x, y);
        game.players.insert(id, Player { dx: 0.0, dy: 0.0 });
        id
    }
    pub fn step(game: &mut Game) {
        for (id, player) in game.players.iter() {
            let pos = game.positions.get_mut(id).expect("Player had no position!");
            pos.x += player.dx;
            pos.y += player.dy;
        }
    }
}

#[derive(Clone)]
pub struct Game {
    id_counter: u64,
    positions: IdHashMap<Position>,
    players: IdHashMap<Player>,
}

type TimeMap<T> = HashMap<u64, T, U64DoNothingBuildHasher>;
fn new_time_map<T>() -> TimeMap<T> {
    TimeMap::<T>::with_hasher(Default::default())
}
fn new_id_hashmap<T>() -> IdHashMap<T> {
    IdHashMap::<T>::with_hasher(Default::default())
}

const PLAYER_VISUAL_WIDTH: i32 = 6;
impl Game {
    pub fn new() -> Self {
        Game {
            id_counter: 0,
            positions: new_id_hashmap(),
            players: new_id_hashmap(),
        }
    }
    pub fn create_game_object(&mut self, x: f64, y: f64) -> GameObjectId {
        let id = GameObjectId(self.id_counter);
        self.id_counter += 1;
        self.positions.insert(id, Position { x, y });
        id
    }
    pub fn step(&mut self) {
        Player::step(self);
    }
    pub fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
        canvas.set_draw_color((255, 255, 255, 255));
        canvas.clear();
        canvas.set_draw_color((0, 0, 255, 255));
        for (id, _player) in self.players.iter() {
            let position = self.positions.get(id).unwrap();
            let rect = Rect::new(
                position.x as i32 - PLAYER_VISUAL_WIDTH / 2,
                position.y as i32 - PLAYER_VISUAL_WIDTH / 2,
                PLAYER_VISUAL_WIDTH as u32,
                PLAYER_VISUAL_WIDTH as u32,
            );
            canvas.fill_rect(rect).unwrap();
        }
        canvas.present();
    }
}

pub struct RollbackableGame {
    pub current_time: u64,
    frames: TimeMap<Game>,
    commands: TimeMap<Vec<(GameObjectId, Command)>>,
}

impl RollbackableGame {
    pub fn new(starting_game: Game) -> Self {
        let mut frames = new_time_map();
        frames.insert(0, starting_game);
        RollbackableGame {
            current_time: 0,
            frames,
            commands: new_time_map(),
        }
    }
    pub fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
        self.current_frame().draw(canvas);
    }
    pub fn current_frame(&self) -> &Game {
        self.frames
            .get(&self.current_time)
            .expect("Current frame not present!")
    }
    pub fn add_command(&mut self, player_id: GameObjectId, command: Command, time: u64) {
        let existing_commands = self.commands.entry(time).or_insert_with(Vec::new);
        existing_commands.push((player_id, command));
    }
    pub fn step(&mut self) {
        let mut next_frame = self.current_frame().clone();
        if let Some(commands) = self.commands.get(&self.current_time) {
            for (player_id, command) in commands {
                command.apply(&mut next_frame, *player_id)
            }
        }
        next_frame.step();
        self.current_time += 1;
        self.frames.insert(self.current_time, next_frame);
    }
}
