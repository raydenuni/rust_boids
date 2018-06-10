//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::audio;
use ggez::conf;
use ggez::event::{self, Keycode, Mod, EventHandler};
use ggez::graphics;
use ggez::graphics::{Vector2};
use ggez::timer;

use std::env;
use std::path;

mod actors;
mod math;
mod boids_mgr;

use actors::{ Actor, ActorType };

/// **********************************************************************
/// So that was the real meat of our game.  Now we just need a structure
/// to contain the images, sounds, etc. that we need to hang on to; this
/// is our "asset management system".  All the file names and such are
/// just hard-coded.
/// **********************************************************************

pub struct Assets {
    player_image: graphics::Image,
    shot_image: graphics::Image,
    rock_image: graphics::Image,
    font: graphics::Font,
    shot_sound: audio::Source,
    hit_sound: audio::Source,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image = graphics::Image::new(ctx, "/player.png")?;
        let shot_image = graphics::Image::new(ctx, "/shot.png")?;
        let rock_image = graphics::Image::new(ctx, "/rock.png")?;
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 18)?;

        let shot_sound = audio::Source::new(ctx, "/pew.ogg")?;
        let hit_sound = audio::Source::new(ctx, "/boom.ogg")?;
        Ok(Assets {
            player_image,
            shot_image,
            rock_image,
            font,
            shot_sound,
            hit_sound,
        })
    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Player => &self.player_image,
            ActorType::Rock => &self.rock_image,
            ActorType::Shot => &self.shot_image,
        }
    }
}

/// **********************************************************************
/// Now we're getting into the actual game loop.  The `MainState` is our
/// game's "global" state, it keeps track of everything we need for
/// actually running the game.
///
/// Our game objects are simply a vector for each actor type, and we
/// probably mingle gameplay-state (like score) and hardware-state
/// (like `gui_dirty`) a little more than we should, but for something
/// this small it hardly matters.
/// **********************************************************************

struct MainState {
    actor_mgr: actors::ActorManager,
    boid_mgr: boids_mgr::BoidComponent,
    level: i32,
    score: i32,
    assets: Assets,
    screen_width: u32,
    screen_height: u32,
    input: actors::InputState,
    player_shot_timeout: f32,
    gui_dirty: bool,
    frames: usize,
    // score_display: graphics::Text,
    // level_display: graphics::Text,
    fps_display: graphics::Text,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        ctx.print_resource_stats();
        graphics::set_background_color(ctx, (0, 0, 0, 255).into());

        println!("Game resource path: {:?}", ctx.filesystem);

        print_instructions();

        let assets = Assets::new(ctx)?;
        // let score_disp = graphics::Text::new(ctx, "score", &assets.font)?;
        // let level_disp = graphics::Text::new(ctx, "level", &assets.font)?;
        let fps_disp = graphics::Text::new(ctx, "fps", &assets.font)?;

        let actor_mgr = actors::ActorManager::new();
        let mut boid_mgr = boids_mgr::BoidComponent::new();
        boid_mgr.init();

        let s = MainState {
            actor_mgr,
            boid_mgr,
            level: 0,
            score: 0,
            assets,
            screen_width: ctx.conf.window_mode.width,
            screen_height: ctx.conf.window_mode.height,
            input: actors::InputState::default(),
            player_shot_timeout: 0.0,
            gui_dirty: true,
            frames: 0,
            // score_display: score_disp,
            // level_display: level_disp,
            fps_display: fps_disp,
        };

        Ok(s)
    }

    fn fire_player_shot(&mut self) {
        self.player_shot_timeout = actors::PLAYER_SHOT_TIME;
        self.actor_mgr.fire_player_shot_helper();
        let _ = self.assets.shot_sound.play();
    }

    fn update_ui(&mut self, ctx: &mut Context) {
        // let score_str = format!("Score: {}", self.score);
        // let score_text = graphics::Text::new(ctx, &score_str, &self.assets.font).unwrap();
        // self.score_display = score_text;
        //
        // let level_str = format!("Level: {}", self.level);
        // let level_text = graphics::Text::new(ctx, &level_str, &self.assets.font).unwrap();
        // self.level_display = level_text;

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
    println!("Welcome to ASTROBLASTO!");
    println!();
    println!("How to play:");
    println!("L/R arrow keys rotate your ship, up thrusts, space bar fires");
    println!();
}

pub fn draw_actor(
    assets: &Assets,
    ctx: &mut Context,
    actor: &Actor,
    world_coords: (u32, u32),
) -> GameResult<()> {
    let (screen_w, screen_h) = world_coords;
    let pos = math::world_to_screen_coords(screen_w, screen_h, &actor.pos);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam {
        dest: pos,
        rotation: actor.facing as f32,
        offset: graphics::Point2::new(0.5, 0.5),
        ..Default::default()
    };
    graphics::draw_ex(ctx, image, drawparams)
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

            // Update the player state based on the user input.
            //player_handle_input(&mut self.player, &self.input, seconds);
            self.player_shot_timeout -= SECONDS;
            if self.input.fire && self.player_shot_timeout < 0.0 {
                self.fire_player_shot();
            }

            // update all the actors
            {
                self.actor_mgr.update(SECONDS, &self.input, self.screen_width as f32, self.screen_height as f32);

                let num_hits = self.actor_mgr.handle_collisions();
                if num_hits > 0 {
                    self.score += num_hits;
                    self.gui_dirty = true;
                    let _ = self.assets.hit_sound.play();
                }

                self.actor_mgr.clear_dead_stuff();

                //self.check_for_level_respawn();
                if self.actor_mgr.rocks_are_empty() {
                    self.level += 1;
                    self.gui_dirty = true;
                    self.actor_mgr.when_rocks_empty(self.level + 5);
                }
            }

            // Using a gui_dirty flag here is a little messy but fine here.
            if self.gui_dirty {
                self.update_ui(ctx);
                self.gui_dirty = false;
            }

            // Finally we check for our end state.
            // I want to have a nice death screen eventually, but for now we just quit.
            if self.actor_mgr.player_is_dead() {
                println!("Game over!");
                let _ = ctx.quit();
            }

            // update boids
            self.boid_mgr.update(SECONDS, Vector2::new(self.screen_width as f32, self.screen_height as f32));
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Our drawing is quite simple.
        // Just clear the screen...
        graphics::clear(ctx);

        // Loop over all objects drawing them...
        {
            let assets = &mut self.assets;
            let coords = (self.screen_width, self.screen_height);
            //self.actor_mgr.draw(ctx, assets, coords)?;
            self.boid_mgr.draw(ctx, assets, coords)?;
        }

        // And draw the GUI elements in the right places.
        // let level_dest = graphics::Point2::new(10.0, 10.0);
        // graphics::draw(ctx, &self.level_display, level_dest, 0.0)?;
        // let score_dest = graphics::Point2::new(200.0, 10.0);
        // graphics::draw(ctx, &self.score_display, score_dest, 0.0)?;
        let fps_dest = graphics::Point2::new(400.0, 10.0);
        graphics::draw(ctx, &self.fps_display, fps_dest, 0.0)?;


        // Then we flip the screen...
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
            Keycode::Up => {
                self.input.yaxis = 1.0;
            }
            Keycode::Left => {
                self.input.xaxis = -1.0;
            }
            Keycode::Right => {
                self.input.xaxis = 1.0;
            }
            Keycode::Space => {
                self.input.fire = true;
            }
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
            Keycode::Up => {
                self.input.yaxis = 0.0;
            }
            Keycode::Left | Keycode::Right => {
                self.input.xaxis = 0.0;
            }
            Keycode::Space => {
                self.input.fire = false;
            }
            _ => (), // Do nothing
        }
    }
}

/// **********************************************************************
/// Finally our main function!  Which merely sets up a config and calls
/// `ggez::event::run()` with our `EventHandler` type.
/// **********************************************************************
pub fn main() {
    let mut cb = ContextBuilder::new("astroblasto", "ggez")
        .window_setup(conf::WindowSetup::default().title("Astroblasto!"))
        .window_mode(conf::WindowMode::default().dimensions(800, 800));

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
