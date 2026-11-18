// Collision resolution and elastic bounce physics for FPV drone.
// Direct port of DroneCollision.java.

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

#[cfg(test)]
mod tests {
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
