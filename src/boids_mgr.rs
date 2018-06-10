
use ggez::graphics::{Point2, Vector2};
use ggez::graphics;
use ggez::{Context, GameResult};
use math;
use rand;

const NUM_BOIDS:usize = 100;
const NUM_ATTRACTORS:usize = 8;

const ACCELERATION_LIMIT: f32 = 60.;
const SPEED_LIMIT: f32 = 100.;
const MIN_SPEED_LIMIT: f32 = 50.;
const _SPEED_LIMIT_SQ:f32 = SPEED_LIMIT * SPEED_LIMIT;

const SEPARATION_DISTANCE:f32 = 40.;
const SEP_DIST_SQ:f32 = SEPARATION_DISTANCE * SEPARATION_DISTANCE;
const COHESION_DISTANCE:f32 = 200.;
const COH_DIST_SQ:f32 = COHESION_DISTANCE * COHESION_DISTANCE;
const ALIGNMENT_DISTANCE:f32 = 200.;
const ALI_DIST_SQ:f32 = ALIGNMENT_DISTANCE * ALIGNMENT_DISTANCE;

const SEPARATION_FORCE:f32 = 10.15;
const COHESION_FORCE:f32 = 0.1;
const ALIGNMENT_FORCE:f32 = 0.25;

const ATTRACTOR_RADIUS:f32 = 150.;
const ATTRACTOR_FORCE:f32 = 0.525;

pub struct Attractors {
    position: Vec<Point2>,
    radius: Vec<f32>,
    force: Vec<f32>,
}

pub struct BoidComponent {
    position: Vec<Point2>,
    acceleration: Vec<Vector2>,
    velocity: Vec<Vector2>,
    attractors: Attractors,
}

impl BoidComponent {

    pub fn new() -> BoidComponent {
        let att = Attractors {
            position: Vec::<Point2>::with_capacity(NUM_ATTRACTORS),
            radius: Vec::<f32>::with_capacity(NUM_ATTRACTORS),
            force: Vec::<f32>::with_capacity(NUM_ATTRACTORS),
        };

        BoidComponent {
            position: Vec::<Point2>::with_capacity(NUM_BOIDS),
            acceleration: Vec::<Vector2>::with_capacity(NUM_BOIDS),
            velocity: Vec::<Vector2>::with_capacity(NUM_BOIDS),
            attractors: att,
        }
    }

    pub fn init(&mut self, screen_size: &Vector2) {
        for _ in 0..NUM_BOIDS {
            self.spawn_random();
        }
        for _ in 0..NUM_ATTRACTORS {
            self.spawn_attractor(screen_size);
        }
    }

    pub fn spawn_attractor(&mut self, screen_size: &Vector2) -> usize {
        self.attractors.position.push(Point2::new(screen_size.x * rand::random::<f32>() - screen_size.x/2f32, screen_size.y * rand::random::<f32>() - screen_size.y/2f32));
        self.attractors.radius.push(ATTRACTOR_RADIUS);
        self.attractors.force.push(ATTRACTOR_FORCE);

        self.attractors.position.len() - 1
    }

    pub fn spawn_random(&mut self) -> usize {
        self.position.push(Point2::new(100. * rand::random::<f32>(), 100. * rand::random::<f32>()));
        self.acceleration.push(Vector2::new(0., 0.));
        self.velocity.push(Vector2::new(400. * rand::random::<f32>() - 200., 400. * rand::random::<f32>() - 200.));
        self.position.len() - 1
    }

    pub fn update(&mut self, dt: f32, screen_size: &Vector2) {
        let boids_length = self.position.len();

        //println!("\n\n--------\nUPDATING BOIDS");
        for b in 0..boids_length {
            //println!("\nboid {}", b);

            let mut s_force = Vector2::new(0., 0.);
            let mut c_force = Vector2::new(0., 0.);
            let mut a_force = Vector2::new(0., 0.);

            for target in 0..self.attractors.position.len() {
                let spare = self.position[b] - self.attractors.position[target];
                //println!("spare: {}", spare);
                let dist = spare.norm();
                //println!("dist: {}", dist);
                //println!("self.attractors.radius[target]: {}", self.attractors.radius[target]);
                if dist < self.attractors.radius[target] {
                    let length = spare.norm();
                    let delta = Vector2::new(self.attractors.force[target] * spare.x / length, self.attractors.force[target] * spare.y / length);
                    self.velocity[b] -= delta;
                }
            }

            for target in 0..boids_length {

                if b == target {
                    continue;
                }

                let spare = self.position[b] - self.position[target];
                let dist_squared = spare.x*spare.x + spare.y*spare.y;

                if dist_squared < SEP_DIST_SQ {
                    let dist = spare.norm();
                    let force = 1. - (SEPARATION_DISTANCE - dist) / SEPARATION_DISTANCE;

                    s_force += spare * force; // * 1000. / spare.norm().powf(2.);
                } else {
                    if dist_squared < COH_DIST_SQ {
                        c_force += spare;
                    }
                    if dist_squared < ALI_DIST_SQ {
                        //println!("alignment: my velocity: [{},{}] -- target velocity: [{},{}]", self.velocity[b].x, self.velocity[b].y, self.velocity[target].x, self.velocity[target].y);
                        a_force += self.velocity[target];
                    }
                }
            }

            // separation
            let sep_length = s_force.len() as f32;
            let sep_vector = Vector2::new(s_force.x * SEPARATION_FORCE / sep_length, s_force.y * SEPARATION_FORCE / sep_length);
            self.acceleration[b] += sep_vector;

            // cohesion
            let coh_length = c_force.len() as f32;
            let coh_vector = Vector2::new(-c_force.x * COHESION_FORCE / coh_length, -c_force.y * COHESION_FORCE / coh_length);
            self.acceleration[b] += coh_vector;

            // alignment
            let ali_length = a_force.len() as f32;
            let ali_vector = Vector2::new(a_force.x * COHESION_FORCE / ali_length, a_force.y * ALIGNMENT_FORCE / ali_length);
            self.acceleration[b] += ali_vector;

            //println!("SEPARATION_FORCE: {} -- s_force.x: {} -- s_force.u: {} -- sep_length: {}", SEPARATION_FORCE, s_force.x, s_force.y, sep_length);
            //println!("[b={}] ---- sep_vector: [{},{}] -- coh_vector: [{},{}] -- ali_vector: [{},{}]", b, sep_vector.x, sep_vector.y, coh_vector.x, coh_vector.y, ali_vector.x, ali_vector.y);
        }

        for i in 0..self.acceleration.len() {
            if self.acceleration[i].norm() > ACCELERATION_LIMIT {
                self.acceleration[i] = self.acceleration[i].normalize() * ACCELERATION_LIMIT;
            }

            self.velocity[i] += self.acceleration[i] * dt;
            if self.velocity[i].norm() > SPEED_LIMIT {
                self.velocity[i] = self.velocity[i].normalize() * SPEED_LIMIT;
            }
            if self.velocity[i].norm() < MIN_SPEED_LIMIT {
                self.velocity[i] = self.velocity[i].normalize() * MIN_SPEED_LIMIT;
            }

            self.position[i] += self.velocity[i] * dt;
            math::wrap_actor_position(&mut self.position[i], &screen_size);
        }
    }

    pub fn draw(&mut self,
                ctx: &mut Context,
                assets: &super::Assets,
                world_coords: (u32, u32)) -> GameResult<()> {
        for b in 0..self.position.len() {
            let (screen_w, screen_h) = world_coords;
            let position = super::math::world_to_screen_coords(screen_w, screen_h, &self.position[b]);
            let image = &assets.player_image;
            let drawparams = graphics::DrawParam {
                dest: position,
                rotation: math::angle_from_vec(&self.velocity[b]),
                offset: graphics::Point2::new(0.5, 0.5),
                ..Default::default()
            };
            graphics::draw_ex(ctx, image, drawparams)?;
        }

        for i in 0..self.attractors.position.len() {
            let (screen_w, screen_h) = world_coords;
            let position = super::math::world_to_screen_coords(screen_w, screen_h, &self.attractors.position[i]);

            graphics::circle(ctx, graphics::DrawMode::Line(1.), position, self.attractors.radius[i], 1f32)?;
        }

        Ok(())
    }
}
