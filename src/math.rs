extern crate rand;

use ggez::graphics::{Point2, Vector2};

/// *********************************************************************
/// Basic stuff, make some helpers for vector functions.
/// ggez includes the nalgebra math library to provide lots of
/// math stuff  We just add some helpers.
/// **********************************************************************

/// Create a unit vector representing the
/// given angle (in radians)
pub fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

pub fn angle_from_vec(vec: &Vector2) -> f32 {
    vec.x.atan2(vec.y)
}

/// Just makes a random `Vector2` with the given max magnitude.
pub fn random_vec(max_magnitude: f32) -> Vector2 {
    let angle = rand::random::<f32>() * 2.0 * ::std::f32::consts::PI;
    let mag = rand::random::<f32>() * max_magnitude;
    vec_from_angle(angle) * (mag)
}

/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
/// pointing downward and the origin at the top-left,
pub fn world_to_screen_coords(screen_width: u32, screen_height: u32, point: &Point2) -> Point2 {
    let width = screen_width as f32;
    let height = screen_height as f32;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Point2::new(x, y)
}

/// Takes an actor and wraps its position to the bounds of the
/// screen, so if it goes off the left side of the screen it
/// will re-enter on the right side and so on.
pub fn wrap_actor_position(pos: &mut Point2, boundary: &Vector2) {
    // Wrap screen
    let screen_x_bounds = boundary.x / 2.0;
    let screen_y_bounds = boundary.y / 2.0;
    if pos.x > screen_x_bounds {
        pos.x -= boundary.x;
    } else if pos.x < -screen_x_bounds {
        pos.x += boundary.x;
    };
    if pos.y > screen_y_bounds {
        pos.y -= boundary.y;
    } else if pos.y < -screen_y_bounds {
        pos.y += boundary.y;
    }
}
