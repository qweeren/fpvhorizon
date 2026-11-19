// Collision resolution and elastic bounce physics for FPV drone.
// Based on DroneCollision.java from Minecraft FPV (GPL-3.0, github.com/minecraft-fpv/mfpv-client).

use super::math::Vec3f;

pub const ELASTIC: f32 = 0.2;
pub const GLANCING_KEEP: f32 = 0.65;
pub const MIN_BOUNCE_SPEED: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionResult {
    pub velocity: Vec3f,
    pub displacement: Vec3f,
}

pub fn resolve(displacement: Vec3f, clipped: Vec3f, pre_collision_velocity: Vec3f) -> CollisionResult {
    let desired_speed = pre_collision_velocity.length();
    let v_clip = clipped.sub(displacement);
    let v_clip_norm = v_clip.normalized_or_zero();
    let disp_norm = displacement.normalized_or_zero();

    let angle = disp_norm.angle_between(v_clip_norm.mul(-1.0));
    let c_proj = clipped.project(v_clip_norm);

    if c_proj.x.is_nan() || c_proj.y.is_nan() || c_proj.z.is_nan() {
        return CollisionResult {
            velocity: Vec3f::ZERO,
            displacement: clipped,
        };
    }

    let h_dir = c_proj.sub(clipped).normalized_or_zero();
    let h_len = v_clip.length() * angle.tan();
    let h_clip = h_dir.mul(h_len);

    let puncture = clipped.add(h_clip);
    let bounce_final = clipped.add(v_clip);
    let bounce_direction = bounce_final.sub(puncture).normalized_or_zero();

    let glancing = angle.sin();
    let out_speed = desired_speed * (glancing * GLANCING_KEEP + (1.0 - glancing) * ELASTIC);

    if out_speed < MIN_BOUNCE_SPEED {
        return CollisionResult {
            velocity: Vec3f::ZERO,
            displacement: clipped,
        };
    }

    CollisionResult {
        velocity: bounce_direction.mul(out_speed),
        displacement: bounce_final,
    }
}

/// Clips `movement` against solid `boxes` for a body with bounds `aabb`, the way vanilla's
/// `Entity.collide` does: Y first, then the larger horizontal axis. Boxes and bounds are
/// `[min_x, min_y, min_z, max_x, max_y, max_z]` in world coordinates. Returns the movement allowed.
pub fn sweep(aabb: [f64; 6], movement: [f64; 3], boxes: &[[f64; 6]]) -> [f64; 3] {
    const EPS: f64 = 1e-7;
    let mut bounds = aabb;
    let mut out = movement;
    let order = if movement[0].abs() < movement[2].abs() { [1, 2, 0] } else { [1, 0, 2] };
    for axis in order {
        let mut d = out[axis];
        if d == 0.0 {
            continue;
        }
        for s in boxes {
            let overlaps = (0..3)
                .filter(|&a| a != axis)
                .all(|a| s[a + 3] > bounds[a] + EPS && s[a] < bounds[a + 3] - EPS);
            if !overlaps {
                continue;
            }
            if d > 0.0 && s[axis] >= bounds[axis + 3] - EPS {
                d = d.min(s[axis] - bounds[axis + 3]);
            } else if d < 0.0 && s[axis + 3] <= bounds[axis] + EPS {
                d = d.max(s[axis + 3] - bounds[axis]);
            }
        }
        out[axis] = d;
        bounds[axis] += d;
        bounds[axis + 3] += d;
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn sweep_stops_on_floor_and_slides_along_it() {
        let body = [-0.3, 1.0, -0.3, 0.3, 2.8, 0.3];
        let floor = [[-5.0, 0.0, -5.0, 5.0, 1.0, 5.0]];
        let out = sweep(body, [0.5, -2.0, 0.0], &floor);
        assert!(out[1].abs() < 1e-9, "should land on the floor, got {:?}", out);
        assert!((out[0] - 0.5).abs() < 1e-9, "should keep sliding along x");
    }

    #[test]
    fn sweep_stops_at_wall() {
        let body = [-0.3, 0.0, -0.3, 0.3, 1.8, 0.3];
        let wall = [[1.0, 0.0, -1.0, 2.0, 3.0, 1.0]];
        let out = sweep(body, [2.0, 0.0, 0.0], &wall);
        assert!((out[0] - 0.7).abs() < 1e-9, "got {:?}", out);
    }

    use super::*;

    #[test]
    fn test_collision_resolve_glancing() {
        let disp = Vec3f::new(1.0, -1.0, 0.0);
        let clipped = Vec3f::new(1.0, 0.0, 0.0);
        let vel = Vec3f::new(10.0, -10.0, 0.0);
        let res = resolve(disp, clipped, vel);
        assert!(res.velocity.length() > 0.0);
        assert!(!res.velocity.x.is_nan());
    }

    #[test]
    fn test_collision_resolve_low_speed() {
        let disp = Vec3f::new(0.01, 0.0, 0.0);
        let clipped = Vec3f::new(0.0, 0.0, 0.0);
        let vel = Vec3f::new(0.01, 0.0, 0.0);
        let res = resolve(disp, clipped, vel);
        assert_eq!(res.velocity, Vec3f::ZERO);
    }
}
