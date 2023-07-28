use std::{
    net::{TcpListener, TcpStream},
    thread::sleep,
    time::{Duration, Instant},
};

use game::{
    commands::{Command, TimedCommand},
    Game, Player, RollbackableGame,
};
use sdl2::keyboard::Keycode;

mod game;
mod network;
use network::net_thread;

const WINDOW_WIDTH: u32 = 400;
const WINDOW_HEIGHT: u32 = 400;
const TICK_TIME: Duration = Duration::from_millis(1000 / 60);
struct KeyState {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}
impl KeyState {
    fn new() -> Self {
        KeyState {
            left: false,
            right: false,
            up: false,
            down: false,
        }
    }
}
const PLAYER_SPEED: f64 = 4.0;
fn generate_move_command(key_state: &KeyState) -> Command {
    let dx = if key_state.left {
        if key_state.right {
            0.0
        } else {
            -PLAYER_SPEED
        }
    } else if key_state.right {
        PLAYER_SPEED
    } else {
        0.0
    };
    let dy = if key_state.down {
        if key_state.up {
            0.0
        } else {
            PLAYER_SPEED
        }
    } else if key_state.up {
        -PLAYER_SPEED
    } else {
        0.0
    };
    Command::MoveByCommand(dx, dy)
}

fn format_usage_message(program_name: &str) -> String {
    format!(
        "Usage: {} [player name] [(host [port]|(client [ip] [port])]",
        program_name
    )
}
fn print_usage_and_quit(program_name: &str) -> ! {
    println!("{}", format_usage_message(program_name));
    std::process::exit(-1);
}

fn main() {
    let mut arguments = std::env::args().into_iter();
    let program_name = arguments
        .next()
        .expect("Expected program name to be passed as first argument");
    let my_name = arguments
        .next()
        .unwrap_or_else(|| print_usage_and_quit(&program_name));
    let host_or_client = arguments
        .next()
        .unwrap_or_else(|| print_usage_and_quit(&program_name));
    let (is_host, connection) = match host_or_client.as_str() {
        "host" => {
            let port = arguments
                .next()
                .unwrap_or_else(|| print_usage_and_quit(&program_name));
            let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                .expect(&format!("Unable to bind to port {}", port));
            let (client, _) = tcp_listener.accept().expect("Unable to accept client");
            (true, client)
        }
        "client" => {
            let ip = arguments
                .next()
                .unwrap_or_else(|| print_usage_and_quit(&program_name));
            let port = arguments
                .next()
                .unwrap_or_else(|| print_usage_and_quit(&program_name));
            let address = format!("{}:{}", ip, port);
            let host =
                TcpStream::connect(&address).expect(&format!("Could not connect to {}", address));
            (false, host)
        }
        other_string => {
            println!("{}", format_usage_message(&program_name));
            panic!("Expect 'host' or 'client', got '{}'", other_string);
        }
    };

    let (their_handshake, set_input_delay, to_other_sender, from_other_receiver) =
        net_thread(is_host, my_name.clone(), connection);

    let mut starting_game = Game::new();
    let player_ids = vec![
        Player::new(&mut starting_game, 100.0, 100.0),
        Player::new(&mut starting_game, 200.0, 100.0),
    ];
    if their_handshake.my_name == my_name {
        panic!("Both players cannot have the same name!");
    }
    let (my_id, their_id) = if their_handshake.my_name < my_name {
        let my_id = player_ids[0];
        let their_id = player_ids[1];
        (my_id, their_id)
    } else {
        let my_id = player_ids[1];
        let their_id = player_ids[0];
        (my_id, their_id)
    };
    let mut game = RollbackableGame::new(starting_game);

    let sdl2_system = sdl2::init().expect("Couldn't initialise SDL");
    let video_subsystem = sdl2_system.video().expect("No video");
    let mut window_builder =
        video_subsystem.window("TIME FLOWS IN ONE DIRECTION", WINDOW_WIDTH, WINDOW_HEIGHT);
    let window = window_builder
        .opengl()
        .build()
        .expect("Could not create window!");
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .expect("Could not create canvas!");
    let mut event_pump = sdl2_system
        .event_pump()
        .expect("Could not obtain event pump!");

    let mut key_state = KeyState::new();

    'main: loop {
        let tick_start = Instant::now();
        let mut moved = false;
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => match keycode {
                    Some(Keycode::W) => {
                        key_state.up = true;
                        moved = true;
                    }
                    Some(Keycode::A) => {
                        key_state.left = true;
                        moved = true;
                    }
                    Some(Keycode::S) => {
                        key_state.down = true;
                        moved = true;
                    }
                    Some(Keycode::D) => {
                        key_state.right = true;
                        moved = true;
                    }
                    _ => {}
                },
                sdl2::event::Event::KeyUp {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => match keycode {
                    Some(Keycode::W) => {
                        key_state.up = false;
                        moved = true;
                    }
                    Some(Keycode::A) => {
                        key_state.left = false;
                        moved = true;
                    }
                    Some(Keycode::S) => {
                        key_state.down = false;
                        moved = true;
                    }
                    Some(Keycode::D) => {
                        key_state.right = false;
                        moved = true;
                    }
                    _ => {}
                },
                sdl2::event::Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => match win_event {
                    sdl2::event::WindowEvent::Close { .. } => {
                        break 'main;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        if moved {
            let command = generate_move_command(&key_state);
            let time = game.current_time + set_input_delay.input_delay;
            let timed_command = TimedCommand {
                time,
                command: command.clone(),
            };
            to_other_sender
                .send(timed_command)
                .expect("Couldn't send command to other player");
            game.add_command(my_id, command, time);
        }
        for timed_command in from_other_receiver.try_iter() {
            if timed_command.time < game.current_time {
                println!("WARNING! Received command too late");
            }
            game.add_command(their_id, timed_command.command, timed_command.time);
        }
        game.draw(&mut canvas);
        game.step();
        let time_passed = tick_start.elapsed();
        if time_passed < TICK_TIME {
            let remaining = TICK_TIME - time_passed;
            sleep(remaining);
        }
    }
}
