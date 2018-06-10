//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf;
use ggez::event::{self, Keycode, Mod, EventHandler};
use ggez::graphics;
use ggez::graphics::{Vector2};
use ggez::timer;

use std::env;
use std::path;

mod math;
mod boids_mgr;

pub struct Assets {
    player_image: graphics::Image,
    font: graphics::Font,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image = graphics::Image::new(ctx, "/player.png")?;
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 18)?;
        Ok(Assets {
            player_image,
            font,
        })
    }
}

struct MainState {
    boid_mgr: boids_mgr::BoidComponent,
    assets: Assets,
    screen_size: Vector2,
    frames: usize,
    fps_display: graphics::Text,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        ctx.print_resource_stats();
        graphics::set_background_color(ctx, (0, 0, 0, 255).into());

        println!("Game resource path: {:?}", ctx.filesystem);

        print_instructions();

        let assets = Assets::new(ctx)?;
        let fps_disp = graphics::Text::new(ctx, "fps", &assets.font)?;

        let mut boid_mgr = boids_mgr::BoidComponent::new();
        let screen_size = Vector2::new(ctx.conf.window_mode.width as f32, ctx.conf.window_mode.height as f32);

        boid_mgr.init(&screen_size);

        let s = MainState {
            boid_mgr,
            assets,
            screen_size,
            frames: 0,
            fps_display: fps_disp,
        };

        Ok(s)
    }

    fn update_ui(&mut self, ctx: &mut Context) {
        self.frames += 1;
        if (self.frames % 100) == 0 {
            let fps_str = format!("FPS: {:.*}", 2, ggez::timer::get_fps(ctx));
            let fps_text = graphics::Text::new(ctx, &fps_str, &self.assets.font).unwrap();
            self.fps_display = fps_text;
        }
    }
}

/// **********************************************************************
/// A couple of utility functions.
/// **********************************************************************

fn print_instructions() {
    println!();
    println!("Welcome to Boids!");
}

/// **********************************************************************
/// Now we implement the `EventHandler` trait from `ggez::event`, which provides
/// ggez with callbacks for updating and drawing our game, as well as
/// handling input events.
/// **********************************************************************
impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            const SECONDS: f32 = 1.0 / (DESIRED_FPS as f32);
            self.update_ui(ctx);
            self.boid_mgr.update(SECONDS, &self.screen_size);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        self.boid_mgr.draw(ctx, &mut self.assets, (self.screen_size.x as u32, self.screen_size.y as u32))?;

        let fps_dest = graphics::Point2::new(400.0, 10.0);
        graphics::draw(ctx, &self.fps_display, fps_dest, 0.0)?;
        graphics::present(ctx);

        // And yield the timeslice
        // This tells the OS that we're done using the CPU but it should get back to this program as soon as it can.
        // This ideally prevents the game from using 100% CPU all the time even if vsync is off.
        // The actual behavior can be a little platform-specific.
        timer::yield_now();
        Ok(())
    }

    // Handle key events.  These just map keyboard events
    // and alter our input state appropriately.
    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::P => {
                let img = graphics::screenshot(ctx).expect("Could not take screenshot");
                img.encode(ctx, graphics::ImageFormat::Png, "/screenshot.png")
                    .expect("Could not save screenshot");
            }
            Keycode::Escape => ctx.quit().unwrap(),
            _ => (), // Do nothing
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            _ => (), // Do nothing
        }
    }
}

/// **********************************************************************
/// Finally our main function!  Which merely sets up a config and calls
/// `ggez::event::run()` with our `EventHandler` type.
/// **********************************************************************
pub fn main() {
    let mut cb = ContextBuilder::new("boids", "ggez")
        .window_setup(conf::WindowSetup::default().title("Boids!"))
        .window_mode(conf::WindowMode::default().dimensions(1200, 1200));

    // We add the CARGO_MANIFEST_DIR/resources to the filesystems paths so
    // we we look in the cargo project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        // We need this re-assignment alas, see
        // https://aturon.github.io/ownership/builders.html
        // under "Consuming builders"
        cb = cb.add_resource_path(path);
    } else {
        println!("Not building from cargo?  Ok.");
    }

    let ctx = &mut cb.build().unwrap();

    match MainState::new(ctx) {
        Err(e) => {
            println!("Could not load game!");
            println!("Error: {}", e);
        }
        Ok(ref mut game) => {
            let result = event::run(ctx, game);
            if let Err(e) = result {
                println!("Error encountered running game: {}", e);
            } else {
                println!("Game exited cleanly.");
            }
        }
    }
}
