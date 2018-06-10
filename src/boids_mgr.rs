
use ggez::graphics::{Point2, Vector2};
use ggez::graphics;
use ggez::{Context, GameResult};
use math;
use rand;

const NUM_BOIDS:usize = 50;

const ACCELERATION_LIMIT: f32 = 30.;
const SPEED_LIMIT: f32 = 100.;
const _SPEED_LIMIT_SQ:f32 = SPEED_LIMIT * SPEED_LIMIT;

const SEPARATION_DISTANCE:f32 = 80.;
const SEP_DIST_SQ:f32 = SEPARATION_DISTANCE * SEPARATION_DISTANCE;
const COHESION_DISTANCE:f32 = 200.;
const COH_DIST_SQ:f32 = COHESION_DISTANCE * COHESION_DISTANCE;
const ALIGNMENT_DISTANCE:f32 = 200.;
const ALI_DIST_SQ:f32 = ALIGNMENT_DISTANCE * ALIGNMENT_DISTANCE;

const SEPARATION_FORCE:f32 = 1.15;
const COHESION_FORCE:f32 = 0.1;
const ALIGNMENT_FORCE:f32 = 0.25;


pub struct BoidComponent {
    position: Vec<Point2>,
    acceleration: Vec<Vector2>,
    velocity: Vec<Vector2>,
}

impl BoidComponent {

    pub fn new() -> BoidComponent {
        BoidComponent {
            position: Vec::<Point2>::with_capacity(NUM_BOIDS),
            acceleration: Vec::<Vector2>::with_capacity(NUM_BOIDS),
            velocity: Vec::<Vector2>::with_capacity(NUM_BOIDS),
        }
    }

    pub fn init(&mut self) {
        for _ in 0..NUM_BOIDS {
            self.spawn_random();
        }
    }

    pub fn spawn_random(&mut self) -> usize {
        self.position.push(Point2::new(100. * rand::random::<f32>(), 100. * rand::random::<f32>()));
        self.acceleration.push(Vector2::new(0., 0.));
        self.velocity.push(Vector2::new(400. * rand::random::<f32>() - 200., 400. * rand::random::<f32>() - 200.));
        self.position.len() - 1
    }

    pub fn update(&mut self, dt: f32, screen_size: Vector2) {
        let boids_length = self.position.len();

        //println!("\n\n--------\nUPDATING BOIDS");
        for b in 0..boids_length {
            //println!("\nboid {}", b);

            let mut s_force = Vector2::new(0., 0.);
            let mut c_force = Vector2::new(0., 0.);
            let mut a_force = Vector2::new(0., 0.);

            for target in 0..boids_length {
                if b == target {
                    continue;
                }

                let spare = self.position[b] - self.position[target];
                let dist_squared = spare.x*spare.x + spare.y*spare.y;

                if dist_squared < SEP_DIST_SQ {
                    s_force += spare * 1000. / spare.norm().powf(2.);
                } else {
                    if dist_squared < COH_DIST_SQ
                    {
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

            self.position[i] += self.velocity[i] * dt;
            math::wrap_actor_position(&mut self.position[i], &screen_size);
        }
    }

    pub fn draw(&mut self,
                ctx: &mut Context,
                assets: &super::Assets,
                world_coords: (u32, u32)) -> GameResult<()> {
        for mut b in 0..self.position.len() {
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
        Ok(())
    }
}
