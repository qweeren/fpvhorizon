// FPV drone flight state machine and frame step orchestration.
// Port of DroneState.java and DroneClientEvents.java.

use super::build::{BuildParam, DroneBuildConfig};
use super::collision;
use super::controller::ControllerConfig;
use super::math::{Quat, Vec3f};
use super::physics::AdvancedPhysicsCore;
use super::physics_settings::PhysicsConfig;
use std::f32::consts::PI;

pub const THROTTLE_RATE: f32 = 0.6;
pub const YAW_RATE_DEG_PER_SEC: f32 = 180.0;
pub const ROLL_PER_MOUSE_DEG: f32 = 1.0;
pub const PITCH_PER_MOUSE_DEG: f32 = 1.0;
pub const HUD_MOUSE_FULL_DEG: f32 = 6.0;
pub const HUD_MOUSE_SMOOTHING: f32 = 0.35;

#[derive(Debug, Clone)]
pub struct DroneState {
    pub active: bool,
    pub orientation: Quat,
    pub position: [f64; 3],
    pub velocity: Vec3f,
    pub throttle: f32,

    pub hud_roll: f32,
    pub hud_pitch: f32,
    pub hud_yaw: f32,
    pub hud_throttle: f32,

    pub hud_mouse_roll: f32,
    pub hud_mouse_pitch: f32,

    pub locked_yaw: f32,
    pub orig_yaw: f32,
    pub orig_pitch: f32,
    pub last_nanos: u64,

    pub physics: AdvancedPhysicsCore,
    pub build: DroneBuildConfig,
    pub physics_config: PhysicsConfig,
    pub controller_config: ControllerConfig,
}

impl Default for DroneState {
    fn default() -> Self {
        Self {
            active: false,
            orientation: Quat::IDENTITY,
            position: [0.0; 3],
            velocity: Vec3f::ZERO,
            throttle: 0.5,
            hud_roll: 0.0,
            hud_pitch: 0.0,
            hud_yaw: 0.0,
            hud_throttle: 0.0,
            hud_mouse_roll: 0.0,
            hud_mouse_pitch: 0.0,
            locked_yaw: 0.0,
            orig_yaw: 0.0,
            orig_pitch: 0.0,
            last_nanos: 0,
            physics: AdvancedPhysicsCore::default(),
            build: DroneBuildConfig::default(),
            physics_config: PhysicsConfig::default(),
            controller_config: ControllerConfig::default(),
        }
    }
}

impl DroneState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn camera_tilt_rad(&self) -> f32 {
        self.build.get(BuildParam::CameraAngle).to_radians()
    }

    pub fn throttle_to_hud(&self, throt: f32) -> f32 {
        if self.controller_config.mode_3d {
            throt
        } else {
            throt * 2.0 - 1.0
        }
    }

    pub fn get_camera_orientation(&self) -> Quat {
        self.orientation.rotate_x(self.camera_tilt_rad())
    }

    pub fn enter(&mut self, pos: [f64; 3], yaw: f32, pitch: f32, now_nanos: u64) {
        self.active = true;
        self.orig_yaw = yaw;
        self.orig_pitch = pitch;
        self.locked_yaw = yaw;

        let yaw_rad = PI - yaw.to_radians();
        let pitch_rad = -pitch.to_radians();
        self.orientation = Quat::rotation_y_x_z(yaw_rad, pitch_rad, 0.0)
            .rotate_x(-self.camera_tilt_rad());

        self.position = pos;
        self.velocity = Vec3f::ZERO;

        let hover = self.physics.get_hover_throttle(&self.build, &self.physics_config);
        self.throttle = hover;
        self.physics.reset(hover, &self.build, &self.physics_config);

        self.hud_yaw = 0.0;
        self.hud_pitch = 0.0;
        self.hud_roll = 0.0;
        self.hud_throttle = self.throttle_to_hud(self.throttle);
        self.hud_mouse_roll = 0.0;
        self.hud_mouse_pitch = 0.0;
        self.last_nanos = now_nanos;
    }

    pub fn exit(&mut self) -> (f32, f32) {
        self.active = false;
        self.velocity = Vec3f::ZERO;
        (self.orig_yaw, self.orig_pitch)
    }

    pub fn step_input_and_physics<F>(&mut self, dt: f32, mouse_dx: f32, mouse_dy: f32, yaw_input: f32, throttle_input: f32, mut mover: F)
    where
        F: FnMut([f64; 3]) -> [f64; 3],
    {
        if !self.active {
            return;
        }

        let dt = dt.clamp(0.0001, 0.1);

        // Throttle update
        let min_throttle = if self.controller_config.mode_3d { -1.0 } else { 0.0 };
        self.throttle = (self.throttle + throttle_input * THROTTLE_RATE * dt).clamp(min_throttle, 1.0);

        // Smooth mouse look for HUD
        self.hud_mouse_roll += ((mouse_dx / HUD_MOUSE_FULL_DEG).clamp(-1.0, 1.0) - self.hud_mouse_roll) * HUD_MOUSE_SMOOTHING;
        self.hud_mouse_pitch += ((mouse_dy / HUD_MOUSE_FULL_DEG).clamp(-1.0, 1.0) - self.hud_mouse_pitch) * HUD_MOUSE_SMOOTHING;

        let hud_yaw = yaw_input;
        let hud_roll = self.hud_mouse_roll;
        let hud_pitch = self.hud_mouse_pitch;
        let hud_throt = self.throttle_to_hud(self.throttle);

        let pitch_deg = mouse_dy * PITCH_PER_MOUSE_DEG;
        let roll_deg = mouse_dx * ROLL_PER_MOUSE_DEG;
        let yaw_deg = yaw_input * YAW_RATE_DEG_PER_SEC * dt;

        // Rotate drone body
        self.orientation = self.orientation
            .rotate_x(-pitch_deg.to_radians())
            .rotate_z(-roll_deg.to_radians())
            .rotate_y(-yaw_deg.to_radians())
            .normalize();

        // Advance aerodynamics & motor simulation
        self.physics.step(dt, self.throttle, self.orientation, &mut self.velocity, &self.build, &self.physics_config);

        // Move player through world
        let requested_movement = [
            (self.velocity.x * dt) as f64,
            (self.velocity.y * dt) as f64,
            (self.velocity.z * dt) as f64,
        ];

        let pre_collision_velocity = self.velocity;
        let clipped_movement = mover(requested_movement);

        let collided = (clipped_movement[0] - requested_movement[0]).abs() > 1e-5
            || (clipped_movement[1] - requested_movement[1]).abs() > 1e-5
            || (clipped_movement[2] - requested_movement[2]).abs() > 1e-5;

        if collided {
            let req_v3 = Vec3f::new(requested_movement[0] as f32, requested_movement[1] as f32, requested_movement[2] as f32);
            let clip_v3 = Vec3f::new(clipped_movement[0] as f32, clipped_movement[1] as f32, clipped_movement[2] as f32);
            let bounce = collision::resolve(req_v3, clip_v3, pre_collision_velocity);

            self.velocity = bounce.velocity;
            let reflected = [
                (bounce.displacement.x as f64) - clipped_movement[0],
                (bounce.displacement.y as f64) - clipped_movement[1],
                (bounce.displacement.z as f64) - clipped_movement[2],
            ];
            let refl_sq = reflected[0] * reflected[0] + reflected[1] * reflected[1] + reflected[2] * reflected[2];
            if refl_sq > 1e-10 {
                let _ = mover(reflected);
            }
        }

        self.hud_yaw = hud_yaw.clamp(-1.0, 1.0);
        self.hud_roll = hud_roll.clamp(-1.0, 1.0);
        self.hud_pitch = hud_pitch.clamp(-1.0, 1.0);
        self.hud_throttle = hud_throt.clamp(-1.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drone_state_lifecycle() {
        let mut drone = DroneState::new();
        assert!(!drone.active);

        drone.enter([100.0, 64.0, 100.0], 90.0, 0.0, 1000);
        assert!(drone.active);
        assert!(drone.throttle > 0.0);

        // Step frame with no movement obstacles
        let mut moved_dist = [0.0; 3];
        drone.step_input_and_physics(0.02, 0.0, 0.0, 0.0, 0.0, |req| {
            moved_dist = req;
            req // perfect free flight, no collisions
        });

        assert_eq!(drone.orig_yaw, 90.0);
        let (y, p) = drone.exit();
        assert_eq!(y, 90.0);
        assert_eq!(p, 0.0);
        assert!(!drone.active);
    }
}
