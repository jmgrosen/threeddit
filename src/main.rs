#![feature(phase)]
#![crate_name = "threeddit"]

extern crate cgmath;
extern crate device;
extern crate freetype;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate piston;
extern crate glfw_game_window;

extern crate native;
extern crate serialize;
extern crate time;

//use sdl2_game_window::GameWindowSDL2;
use glfw_game_window::GameWindowGLFW;
//use piston::input;
//use piston::cam;
use piston::{AssetStore, GameIterator, GameIteratorSettings, GameWindowSettings, Render};
//use piston::vecmath::{vec3_add, vec3_scale, vec3_normalized};

use render::Renderer;

mod render;

fn main() {
    let ref mut window = GameWindowGLFW::new(
        piston::shader_version::opengl::OpenGL_3_3,
        GameWindowSettings {
            title: "Threeddit!".to_string(),
            size: [854, 480],
            fullscreen: false,
            exit_on_esc: true,
        }
        );

    let assets = AssetStore::from_folder("../assets");

    let mut renderer = Renderer::new(assets, window.gfx());

    let mut game_iter = GameIterator::new(
        window,
        &GameIteratorSettings {
            updates_per_second: 150,
            max_frames_per_second: 75,
        }
        );
    let mut t = 0.0;
    let mut last_render = time::precise_time_ns();
    let mut fps_counter = piston::FPSCounter::new();
    for e in game_iter {
        match e {
            Render(_args) => {
                renderer.render(t);
                // t += 1.0/75.0;
                let now = time::precise_time_ns();
                let dt = (now - last_render) as f32 / 1_000_000_000.0f32;
                last_render = now;
                t += dt;
                println!("fps = {}", fps_counter.tick());
            },
            _ => {},
        }
    }
}
