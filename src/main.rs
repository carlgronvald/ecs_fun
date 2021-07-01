#![allow(dead_code)]

use std::{sync::mpsc};
extern crate bitflags;
extern crate nalgebra_glm as glm;

mod window;
mod logic;
mod event;
mod graphics;
mod utils;
use event::Event;
use logic::CompFlag;
use lazy_static::lazy_static;
use rand::Rng;

use crate::logic::{Position, Velocity};

lazy_static! {
    static ref ASSETS_PATH: Box<std::path::Path> = {
        let path = std::env::current_dir().expect("Both executable path and working directory are unavailable.");

        let root_path = path.join("../..");
        let result = if root_path.join("Cargo.toml").is_file() {
            if root_path.join("assets").is_dir() {
                root_path.join("assets")
            } else {
                panic!("No assets directory at project root.");
            }
        } else {
            if path.join("../assets").is_dir() {
                path.join("../assets")
            } else if path.join("assets").is_dir() {
                path.join("assets")
            } else {
                panic!("Either the assets directory is missing or it is inaccessable.")
            }
        };
        result.into_boxed_path()
    };
}

fn main() {
    println!("Hello, world!");


    
    let (window_tx, game_rx) = mpsc::channel::<event::Event>(); 
    let (mut game_graphics_tx, window_graphics_rx) = mpsc::sync_channel::<>(1);

    let window  = unsafe {window::Window::new(window_graphics_rx) };
    let mut game = logic::Game::new();
    let mut rng = rand::thread_rng();
    for i in 0..10000 {
        game.add_entity(CompFlag::all());
        game.positions[i] = Position {
            x : rng.gen::<f32>()*2.0-1.0,
            y : rng.gen::<f32>()*2.0-1.0
        };
        game.velocities[i] = Velocity {
            x : rng.gen::<f32>()/100.0-1.0/200.0,
            y : rng.gen::<f32>()/100.0-1.0/200.0
        }
    }
    game.positions[1] = Position {
        x : 0.5, y : 1.5
    };
    game.positions[2] = Position {
        x : 2.5, y : 1.5
    };
    game.velocities[2] = Velocity {
        x : 0.1, y : 0.1
    };

    let mut i = 0;
    let mut now = std::time::Instant::now();
    let _ = std::thread::spawn(move || {

        loop {
            game.update(&mut game_graphics_tx);
            i += 1;
            if now.elapsed().as_secs() >= 1 {
                println!("{} rounds!", i);
                now = std::time::Instant::now();
                i = 0;
            }
        }
    });

    let eh : window::EventHandler = Box::new(
        move |ev| {
            let send = match ev {
                _ => None
            };
            if let Some(event) = send {
                let _ = window_tx.send(event);
            }
            ()
        }
    );
    unsafe { window.run(eh) };

}
