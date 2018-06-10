extern crate ggez;
extern crate rand;

use ggez::{Context, GameResult};
use ggez::graphics::{Point2, Vector2};
use ggez::nalgebra as na;
use math;

/// *********************************************************************
/// Now we define our Actor's.
/// An Actor is anything in the game world.
/// We're not *quite* making a real entity-component system but it's
/// pretty close.  For a more complicated game you would want a
/// real ECS, but for this it's enough to say that all our game objects
/// contain pretty much the same data.
/// **********************************************************************
#[derive(Debug)]
pub enum ActorType {
    Player,
    Rock,
    Shot,
}

#[derive(Debug)]
pub struct Actor {
    pub tag: ActorType,
    pub pos: Point2,
    pub facing: f32,
    pub velocity: Vector2,
    pub ang_vel: f32,
    pub bbox_size: f32,

    // I am going to lazily overload "life" with a double meaning:
    // for shots, it is the time left to live, for players and rocks, it is the actual hit points.
    pub life: f32,
}

const PLAYER_LIFE: f32 = 1.0;
const SHOT_LIFE: f32 = 2.0;
const ROCK_LIFE: f32 = 1.0;

const PLAYER_BBOX: f32 = 12.0;
const ROCK_BBOX: f32 = 12.0;
const SHOT_BBOX: f32 = 6.0;

const MAX_ROCK_VEL: f32 = 50.0;

pub struct ActorManager {
    player: Actor,
    shots: Vec<Actor>,
    rocks: Vec<Actor>,
}

impl ActorManager {
    pub fn new() -> Self {
        let player = create_player();
        let shots = Vec::new();
        let rocks = create_rocks(0, player.pos, 100.0, 250.0);

        ActorManager {
            player,
            shots,
            rocks,
        }
    }

    pub fn update(&mut self, seconds: f32, input: &InputState, screen_width: f32, screen_height: f32) {
        // Update the player state based on the user input.
        fn player_thrust(actor: &mut Actor, dt: f32) {
            let direction_vector = math::vec_from_angle(actor.facing);
            let thrust_vector = direction_vector * (PLAYER_THRUST);
            actor.velocity += thrust_vector * (dt);
        }
        fn player_handle_input(actor: &mut Actor, input: &InputState, dt: f32) {
            actor.facing += dt * PLAYER_TURN_RATE * input.xaxis;

            if input.yaxis > 0.0 {
                player_thrust(actor, dt);
            }
        }
        player_handle_input(&mut self.player, &input, seconds);

        // Update the physics for all actors.
        {
            fn update_actor_position(actor: &mut Actor, dt: f32) {
                // Clamp the velocity to the max efficiently
                let norm_sq = actor.velocity.norm_squared();
                if norm_sq > MAX_PHYSICS_VEL.powi(2) {
                    actor.velocity = actor.velocity / norm_sq.sqrt() * MAX_PHYSICS_VEL;
                }
                let dv = actor.velocity * (dt);
                actor.pos += dv;
                actor.facing += actor.ang_vel;
            }
            /// Takes an actor and wraps its position to the bounds of the
            /// screen, so if it goes off the left side of the screen it
            /// will re-enter on the right side and so on.
            fn wrap_actor_position(actor: &mut Actor, sx: f32, sy: f32) {
                // Wrap screen
                let screen_x_bounds = sx / 2.0;
                let screen_y_bounds = sy / 2.0;
                if actor.pos.x > screen_x_bounds {
                    actor.pos.x -= sx;
                } else if actor.pos.x < -screen_x_bounds {
                    actor.pos.x += sx;
                };
                if actor.pos.y > screen_y_bounds {
                    actor.pos.y -= sy;
                } else if actor.pos.y < -screen_y_bounds {
                    actor.pos.y += sy;
                }
            }

            // First the player...
            update_actor_position(&mut self.player, seconds);
            wrap_actor_position(
                &mut self.player,
                screen_width as f32,
                screen_height as f32,
            );

            // Then the shots...
            for act in &mut self.shots {
                update_actor_position(act, seconds);
                wrap_actor_position(act, screen_width as f32, screen_height as f32);
                //handle_timed_life
                act.life -= seconds
            }

            // And finally the rocks.
            for act in &mut self.rocks {
                update_actor_position(act, seconds);
                wrap_actor_position(act, screen_width as f32, screen_height as f32);
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, assets: &super::Assets, coords: (u32, u32)) -> GameResult<()> {
        let p = &self.player;
        super::draw_actor(assets, ctx, p, coords)?;

        for s in &self.shots {
            super::draw_actor(assets, ctx, s, coords)?;
        }

        for r in &self.rocks {
            super::draw_actor(assets, ctx, r, coords)?;
        }

        Ok(())
    }

    pub fn rocks_are_empty(&mut self) -> bool {
        self.rocks.is_empty()
    }

    pub fn when_rocks_empty(&mut self, _new_level: i32) {
        //let r = create_rocks(new_level, self.player.pos, 100.0, 250.0);
        //self.rocks.extend(r);
    }

    pub fn player_is_dead(&mut self) -> bool {
        self.player.life <= 0.0
    }

    pub fn fire_player_shot_helper(&mut self) {
        let player: &Actor = &self.player;

        let player = &player;
        let mut shot = create_shot();
        shot.pos = player.pos;
        shot.facing = player.facing;
        let direction = math::vec_from_angle(shot.facing);
        shot.velocity.x = SHOT_SPEED * direction.x;
        shot.velocity.y = SHOT_SPEED * direction.y;

        self.shots.push(shot);
    }

    pub fn clear_dead_stuff(&mut self) {
        self.shots.retain(|s| s.life > 0.0);
        self.rocks.retain(|r| r.life > 0.0);
    }

    pub fn handle_collisions(&mut self) -> i32 {
        let mut num_hits = 0;
        for rock in &mut self.rocks {
            let pdistance = rock.pos - self.player.pos;
            if pdistance.norm() < (self.player.bbox_size + rock.bbox_size) {
                self.player.life = 0.0;
            }
            for shot in &mut self.shots {
                let distance = shot.pos - rock.pos;
                if distance.norm() < (shot.bbox_size + rock.bbox_size) {
                    shot.life = 0.0;
                    rock.life = 0.0;
                    num_hits += 1;
                }
            }
        }
        num_hits
    }
}

/// *********************************************************************
/// Now we have some constructor functions for different game objects.
/// **********************************************************************

pub fn create_player() -> Actor {
    Actor {
        tag: ActorType::Player,
        pos: Point2::origin(),
        facing: 0.,
        velocity: na::zero(),
        ang_vel: 0.,
        bbox_size: PLAYER_BBOX,
        life: PLAYER_LIFE,
    }
}

pub fn create_rock() -> Actor {
    Actor {
        tag: ActorType::Rock,
        pos: Point2::origin(),
        facing: 0.,
        velocity: na::zero(),
        ang_vel: 0.,
        bbox_size: ROCK_BBOX,
        life: ROCK_LIFE,
    }
}

pub fn create_shot() -> Actor {
    Actor {
        tag: ActorType::Shot,
        pos: Point2::origin(),
        facing: 0.,
        velocity: na::zero(),
        ang_vel: SHOT_ANG_VEL,
        bbox_size: SHOT_BBOX,
        life: SHOT_LIFE,
    }
}

/// Create the given number of rocks.
/// Makes sure that none of them are within the
/// given exclusion zone (nominally the player)
/// Note that this *could* create rocks outside the
/// bounds of the playing field, so it should be
/// called before `wrap_actor_position()` happens.
pub fn create_rocks(num: i32, exclusion: Point2, min_radius: f32, max_radius: f32) -> Vec<Actor> {
    assert!(max_radius > min_radius);
    let new_rock = |_| {
        let mut rock = create_rock();
        let r_angle = rand::random::<f32>() * 2.0 * ::std::f32::consts::PI;
        let r_distance = rand::random::<f32>() * (max_radius - min_radius) + min_radius;
        rock.pos = exclusion + math::vec_from_angle(r_angle) * r_distance;
        rock.velocity = math::random_vec(MAX_ROCK_VEL);
        rock
    };
    (0..num).map(new_rock).collect()
}

/// *********************************************************************
/// Now we make functions to handle physics.  We do simple Newtonian
/// physics (so we do have inertia), and cap the max speed so that we
/// don't have to worry too much about small objects clipping through
/// each other.
///
/// Our unit of world space is simply pixels, though we do transform
/// the coordinate system so that +y is up and -y is down.
/// **********************************************************************

pub const SHOT_SPEED: f32 = 200.0;
const SHOT_ANG_VEL: f32 = 0.1;

// Acceleration in pixels per second.
const PLAYER_THRUST: f32 = 100.0;
// Rotation in radians per second.
const PLAYER_TURN_RATE: f32 = 3.0;
// Seconds between shots
pub const PLAYER_SHOT_TIME: f32 = 0.5;

const MAX_PHYSICS_VEL: f32 = 250.0;

/// **********************************************************************
/// The `InputState` is exactly what it sounds like, it just keeps track of
/// the user's input state so that we turn keyboard events into something
/// state-based and device-independent.
/// **********************************************************************
#[derive(Debug)]
pub struct InputState {
    pub xaxis: f32,
    pub yaxis: f32,
    pub fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fire: false,
        }
    }
}
