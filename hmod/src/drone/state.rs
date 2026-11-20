// The drone in the world: spawned on the ground, armed, flown, crashed and reset.
// The flight model is based on DroneState.java and DroneClientEvents.java from Minecraft FPV (GPL-3.0, github.com/minecraft-fpv/mfpv-client).

use super::build::{BuildParam, DroneBuildConfig};
use super::collision;
use super::controller::{ControlAxis, ControllerConfig, FlightMode};
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
/// Motor idle while armed (Betaflight's dynamic idle keeps the props turning).
pub const ARMED_IDLE: f32 = 0.04;
/// The throttle at or under which the drone may arm.
pub const ARM_THROTTLE: f32 = 0.05;
/// Horizon mode stops levelling as the drone turns past this tilt, degrees, so flips and
/// rolls go through (Betaflight's `horizon_limit_degrees`).
pub const HORIZON_LEVEL_TILT_LIMIT: f32 = 135.0;

#[derive(Debug, Clone)]
pub struct DroneState {
    /// The drone is out in the world.
    pub active: bool,
    pub armed: bool,
    /// Hit something too hard: disarmed until reset.
    pub crashed: bool,
    /// Resting on something below it.
    pub grounded: bool,
    pub orientation: Quat,
    /// The centre of the drone's box bottom, in blocks.
    pub position: [f64; 3],
    /// `position` a tick ago, to draw between ticks.
    pub prev_position: [f64; 3],
    pub velocity: Vec3f,
    /// The throttle the pilot commands (0..1, or -1..1 in 3D mode).
    pub throttle: f32,
    /// Where it was put down, and its heading there: reset goes back to it.
    pub launch: [f64; 3],
    pub launch_yaw: f32,
    /// Seconds armed since the last reset.
    pub flight_time: f32,
    /// Half its width (props included) and its height, blocks.
    pub half_width: f64,
    pub height: f64,
    /// The speed change, m/s, that breaks it.
    pub crash_speed: f32,

    pub hud_roll: f32,
    pub hud_pitch: f32,
    pub hud_yaw: f32,
    pub hud_throttle: f32,
    pub hud_mouse_roll: f32,
    pub hud_mouse_pitch: f32,

    pub physics: AdvancedPhysicsCore,
    pub build: DroneBuildConfig,
    pub physics_config: PhysicsConfig,
    pub controller_config: ControllerConfig,
}

impl Default for DroneState {
    fn default() -> Self {
        Self {
            active: false,
            armed: false,
            crashed: false,
            grounded: false,
            orientation: Quat::IDENTITY,
            position: [0.0; 3],
            prev_position: [0.0; 3],
            velocity: Vec3f::ZERO,
            throttle: 0.0,
            launch: [0.0; 3],
            launch_yaw: 0.0,
            flight_time: 0.0,
            half_width: 0.15,
            height: 0.08,
            crash_speed: 9.0,
            hud_roll: 0.0,
            hud_pitch: 0.0,
            hud_yaw: 0.0,
            hud_throttle: -1.0,
            hud_mouse_roll: 0.0,
            hud_mouse_pitch: 0.0,
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

    pub fn flight_mode(&self) -> FlightMode {
        self.controller_config.flight_mode
    }

    fn is_3d(&self) -> bool {
        self.controller_config.flight_mode.is_3d()
    }

    pub fn throttle_to_hud(&self, throt: f32) -> f32 {
        if self.is_3d() {
            throt
        } else {
            throt * 2.0 - 1.0
        }
    }

    pub fn get_camera_orientation(&self) -> Quat {
        self.orientation.rotate_x(self.camera_tilt_rad())
    }

    /// The drone level, facing Minecraft yaw `yaw` (degrees).
    fn level(yaw: f32) -> Quat {
        Quat::rotation_y_x_z(PI - yaw.to_radians(), 0.0, 0.0)
    }

    /// Puts the drone down, disarmed with a full pack, at `pos` (its box's bottom centre)
    /// facing Minecraft yaw `yaw`. That is its launch spot.
    pub fn spawn(&mut self, pos: [f64; 3], yaw: f32) {
        self.launch = pos;
        self.launch_yaw = yaw;
        self.active = true;
        self.reset();
    }

    /// Back to the launch spot, disarmed, uncrashed, with a fresh pack.
    pub fn reset(&mut self) {
        self.armed = false;
        self.crashed = false;
        self.grounded = true;
        self.position = self.launch;
        self.prev_position = self.launch;
        self.orientation = Self::level(self.launch_yaw);
        self.velocity = Vec3f::ZERO;
        self.throttle = 0.0;
        self.flight_time = 0.0;
        self.physics.reset(0.0, &self.build, &self.physics_config);
        self.physics.new_pack();
        self.hud_yaw = 0.0;
        self.hud_pitch = 0.0;
        self.hud_roll = 0.0;
        self.hud_throttle = self.throttle_to_hud(0.0);
        self.hud_mouse_roll = 0.0;
        self.hud_mouse_pitch = 0.0;
    }

    pub fn despawn(&mut self) {
        self.active = false;
        self.armed = false;
        self.velocity = Vec3f::ZERO;
    }

    /// Arms if the throttle is at idle and it isn't crashed. Returns whether it is armed.
    pub fn arm(&mut self) -> bool {
        let idle = if self.is_3d() { self.throttle.abs() } else { self.throttle };
        if self.active && !self.crashed && idle <= ARM_THROTTLE {
            self.armed = true;
        }
        self.armed
    }

    pub fn disarm(&mut self) {
        self.armed = false;
    }

    /// The camera as Minecraft angles: yaw and pitch in degrees, and roll in radians
    /// (positive rolls the view clockwise as the pilot sees it).
    pub fn camera_angles(&self) -> (f32, f32, f32) {
        let [yaw, pitch, roll] = self.get_camera_orientation().to_euler_angles_yxz();
        // `level` builds the orientation as rotationYXZ(PI - yaw, ..): undo that.
        (180.0 - yaw.to_degrees(), -pitch.to_degrees(), roll)
    }

    /// The drone's heading as a Minecraft yaw, degrees.
    pub fn heading(&self) -> f32 {
        let [yaw, _, _] = self.orientation.to_euler_angles_yxz();
        180.0 - yaw.to_degrees()
    }

    /// Where the drone is `partial` of a tick past the last one.
    pub fn position_at(&self, partial: f32) -> [f64; 3] {
        let t = f64::from(partial.clamp(0.0, 1.0));
        std::array::from_fn(|i| self.prev_position[i] + (self.position[i] - self.prev_position[i]) * t)
    }

    /// One rendered frame: read the sticks, turn the drone and set the throttle.
    /// Runs every frame so the camera is as smooth as the frame rate; the physics runs per tick.
    pub fn frame(&mut self, dt: f32, input: &FrameInput) {
        if !self.active {
            return;
        }
        let dt = dt.clamp(0.0, 0.1);

        self.hud_mouse_roll += ((input.mouse_dx / HUD_MOUSE_FULL_DEG).clamp(-1.0, 1.0) - self.hud_mouse_roll) * HUD_MOUSE_SMOOTHING;
        self.hud_mouse_pitch += ((input.mouse_dy / HUD_MOUSE_FULL_DEG).clamp(-1.0, 1.0) - self.hud_mouse_pitch) * HUD_MOUSE_SMOOTHING;

        let mut hud_yaw = input.key_yaw;
        let mut hud_roll = self.hud_mouse_roll;
        let mut hud_pitch = self.hud_mouse_pitch;
        let mut pitch_deg = input.mouse_dy * PITCH_PER_MOUSE_DEG;
        let mut roll_deg = input.mouse_dx * ROLL_PER_MOUSE_DEG;
        let mut yaw_deg = input.key_yaw * YAW_RATE_DEG_PER_SEC * dt;
        // The roll and pitch "sticks" the self-levelling modes read: the pad's, or the mouse's
        // smoothed speed (moving the mouse tilts, letting it rest levels).
        let mut level_sticks = (self.hud_mouse_roll, self.hud_mouse_pitch);

        let mode = self.flight_mode();
        let min_throttle = if mode.is_3d() { -1.0 } else { 0.0 };
        if let Some(sticks) = input.sticks {
            // The sticks command rotation rates (Betaflight rates), the throttle stick sets the
            // throttle directly.
            let cfg = &self.controller_config;
            let (roll, pitch) = (cfg.shape_stick(sticks.roll), cfg.shape_stick(sticks.pitch));
            roll_deg += cfg.rate_deg_per_sec(ControlAxis::Roll, roll) * dt;
            pitch_deg += cfg.rate_deg_per_sec(ControlAxis::Pitch, pitch) * dt;
            yaw_deg += cfg.rate_deg_per_sec(ControlAxis::Yaw, cfg.shape_stick(sticks.yaw)) * dt;
            level_sticks = (roll, pitch);
            hud_roll += sticks.roll;
            hud_pitch += sticks.pitch;
            hud_yaw += sticks.yaw;
            let stick = sticks.throttle.clamp(-1.0, 1.0);
            self.throttle = if mode.is_3d() {
                stick
            } else if cfg.throttle_centre_hover {
                // A gamepad stick springs back to centre: make centre hover, so letting go
                // holds altitude, full up is full throttle and full down is idle.
                // The stick throttle that comes out of the ESCs as hover, idle added.
                let hover = ((self.physics.hover() - ARMED_IDLE) / (1.0 - ARMED_IDLE)).max(0.0);
                if stick < 0.0 { hover * (1.0 + stick) } else { hover + (1.0 - hover) * stick }
            } else {
                (stick + 1.0) * 0.5
            };
        } else if self.armed {
            self.throttle = (self.throttle + input.key_throttle * THROTTLE_RATE * dt).clamp(min_throttle, 1.0);
        } else {
            // Disarmed, the keyboard throttle sits at idle, ready to arm.
            self.throttle = 0.0;
        }

        match mode {
            FlightMode::Acro | FlightMode::Acro3d => {}
            FlightMode::Angle => {
                // The sticks set the tilt, not a rate: their roll and pitch turn is replaced.
                let (roll, pitch) = self.level_step(level_sticks, dt);
                roll_deg = roll;
                pitch_deg = pitch;
            }
            FlightMode::Horizon => {
                // Acro's rates plus levelling that fades out as the sticks go to full, and past
                // a steep tilt, so a full stick flips.
                let deflection = level_sticks.0.abs().max(level_sticks.1.abs()).min(1.0);
                let tilt = self.tilt_deg();
                let past = ((tilt - HORIZON_LEVEL_TILT_LIMIT) / (180.0 - HORIZON_LEVEL_TILT_LIMIT)).clamp(0.0, 1.0);
                let weight = (1.0 - deflection) * (1.0 - past);
                let (roll, pitch) = self.level_step(level_sticks, dt);
                roll_deg += roll * weight;
                pitch_deg += pitch * weight;
            }
        }

        if self.armed {
            self.orientation = self.orientation
                .rotate_x(-pitch_deg.to_radians())
                .rotate_z(-roll_deg.to_radians())
                .rotate_y(-yaw_deg.to_radians())
                .normalize();
        }

        self.hud_yaw = hud_yaw.clamp(-1.0, 1.0);
        self.hud_roll = hud_roll.clamp(-1.0, 1.0);
        self.hud_pitch = hud_pitch.clamp(-1.0, 1.0);
        self.hud_throttle = self.throttle_to_hud(self.throttle).clamp(-1.0, 1.0);
    }

    /// The drone's roll and pitch, degrees, signed as the sticks turn it: right roll and nose
    /// down are positive.
    pub fn attitude_deg(&self) -> (f32, f32) {
        let [_, pitch, roll] = self.orientation.to_euler_angles_yxz();
        (-roll.to_degrees(), -pitch.to_degrees())
    }

    /// How far the drone's top is tipped from straight up, degrees (0 level, 180 upside down).
    pub fn tilt_deg(&self) -> f32 {
        let up = self.orientation.transform_vec3(Vec3f::UP);
        up.y.clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// The self-levelling turn this frame, degrees of (roll, pitch): toward the tilt the sticks
    /// ask for (`sticks` × the angle limit). Betaflight turns the angle error into a rate,
    /// error × strength / 10 per second; this takes that as an exponential approach, so a
    /// long frame or a high strength can't overshoot.
    fn level_step(&self, sticks: (f32, f32), dt: f32) -> (f32, f32) {
        let cfg = &self.controller_config;
        let (roll, pitch) = self.attitude_deg();
        let target = (sticks.0.clamp(-1.0, 1.0) * cfg.angle_limit, sticks.1.clamp(-1.0, 1.0) * cfg.angle_limit);
        let k = 1.0 - (-cfg.level_strength / 10.0 * dt).exp();
        let wrap = |a: f32| (a + 540.0).rem_euclid(360.0) - 180.0;
        (wrap(target.0 - roll) * k, wrap(target.1 - pitch) * k)
    }

    /// One physics step of `dt` seconds: motors and aerodynamics, then the move. `mover` gets the
    /// drone's position and a movement and returns the part of it the world allows.
    pub fn physics_tick<F>(&mut self, dt: f32, mut mover: F)
    where
        F: FnMut([f64; 3], [f64; 3]) -> [f64; 3],
    {
        self.prev_position = self.position;
        if !self.active {
            return;
        }
        let dt = dt.clamp(0.0001, 0.1);
        if self.armed {
            self.flight_time += dt;
        }

        let output = if self.armed {
            let t = self.throttle;
            t.signum() * (ARMED_IDLE + (1.0 - ARMED_IDLE) * t.abs())
        } else {
            0.0
        };
        self.physics.step(dt, output, self.orientation, &mut self.velocity, &self.build, &self.physics_config);

        let requested = [
            f64::from(self.velocity.x * dt),
            f64::from(self.velocity.y * dt),
            f64::from(self.velocity.z * dt),
        ];
        let pre_collision_velocity = self.velocity;
        let clipped = mover(self.position, requested);
        self.add(clipped);

        let collided = (0..3).any(|i| (clipped[i] - requested[i]).abs() > 1e-5);
        self.grounded = requested[1] < 0.0 && clipped[1] > requested[1] + 1e-6;

        if collided {
            let req = Vec3f::new(requested[0] as f32, requested[1] as f32, requested[2] as f32);
            let clip = Vec3f::new(clipped[0] as f32, clipped[1] as f32, clipped[2] as f32);
            let bounce = collision::resolve(req, clip, pre_collision_velocity);
            let impact = pre_collision_velocity.sub(bounce.velocity).length();
            if self.armed && impact > self.crash_speed {
                self.crashed = true;
                self.armed = false;
            }
            self.velocity = bounce.velocity;
            let reflected = [
                f64::from(bounce.displacement.x) - clipped[0],
                f64::from(bounce.displacement.y) - clipped[1],
                f64::from(bounce.displacement.z) - clipped[2],
            ];
            if reflected.iter().map(|v| v * v).sum::<f64>() > 1e-10 {
                let moved = mover(self.position, reflected);
                self.add(moved);
            }
        }

        if self.grounded && !self.armed {
            // Sitting on the ground: it stops sliding and settles level on its heading.
            self.velocity = Vec3f::new(self.velocity.x * 0.5, self.velocity.y.max(0.0), self.velocity.z * 0.5);
            self.orientation = self.orientation.slerp(Self::level(self.heading()), 0.5).normalize();
        }
    }

    fn add(&mut self, movement: [f64; 3]) {
        for (p, m) in self.position.iter_mut().zip(movement) {
            *p += m;
        }
    }
}

/// The pilot's inputs over one frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameInput {
    /// Mouse turn this frame, in degrees (Minecraft's yaw and pitch turn).
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    /// Keyboard yaw (A/D, arrows) and throttle (W/S, arrows), -1..1.
    pub key_yaw: f32,
    pub key_throttle: f32,
    /// A gamepad's sticks, when one is flying.
    pub sticks: Option<Sticks>,
}

/// Gamepad sticks, each -1..1 with right and up positive.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Sticks {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub throttle: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn free(_: [f64; 3], m: [f64; 3]) -> [f64; 3] {
        m
    }

    /// A floor at y = 0.
    fn floor(pos: [f64; 3], m: [f64; 3]) -> [f64; 3] {
        [m[0], m[1].max(-pos[1]), m[2]]
    }

    #[test]
    fn spawns_disarmed_and_level_and_resets_to_launch() {
        let mut drone = DroneState::new();
        drone.spawn([10.0, 64.0, 5.0], 90.0);
        assert!(drone.active && !drone.armed);
        let (yaw, pitch, roll) = drone.camera_angles();
        let dy = (yaw - 90.0 + 540.0).rem_euclid(360.0) - 180.0;
        assert!(dy.abs() < 0.01, "faces where the player looked, got {yaw}");
        assert!((pitch + drone.build.get(BuildParam::CameraAngle)).abs() < 0.01, "camera tilted up, got {pitch}");
        assert!(roll.abs() < 0.001);
        drone.position = [0.0; 3];
        drone.reset();
        assert_eq!(drone.position, [10.0, 64.0, 5.0]);
    }

    #[test]
    fn disarmed_drone_sits_on_the_ground() {
        let mut drone = DroneState::new();
        drone.spawn([0.0, 0.0, 0.0], 0.0);
        for _ in 0..40 {
            drone.frame(0.05, &FrameInput { key_throttle: 1.0, ..Default::default() });
            drone.physics_tick(0.05, floor);
        }
        assert!(drone.position[1].abs() < 1e-6, "stayed down, at {}", drone.position[1]);
        assert!(drone.grounded);
    }

    #[test]
    fn arms_only_at_idle_and_climbs_when_armed() {
        let mut drone = DroneState::new();
        drone.spawn([0.0; 3], 0.0);
        drone.throttle = 0.5;
        assert!(!drone.arm(), "won't arm with throttle up");
        drone.throttle = 0.0;
        assert!(drone.arm());
        // The keyboard throttle ramps up at THROTTLE_RATE: two seconds of W.
        for _ in 0..40 {
            drone.frame(0.05, &FrameInput { key_throttle: 1.0, ..Default::default() });
            drone.physics_tick(0.05, floor);
        }
        assert!(drone.position[1] > 3.0, "climbed to {}", drone.position[1]);
        assert!(drone.flight_time > 1.9);
    }

    #[test]
    fn a_hard_hit_crashes_and_disarms() {
        let mut drone = DroneState::new();
        drone.spawn([0.0, 10.0, 0.0], 0.0);
        drone.arm();
        drone.velocity = Vec3f::new(0.0, -30.0, 0.0);
        for _ in 0..10 {
            drone.physics_tick(0.05, floor);
        }
        assert!(drone.crashed && !drone.armed);
        assert!(!drone.arm(), "a crashed drone stays disarmed until reset");
        drone.reset();
        assert!(drone.arm());
    }

    #[test]
    fn hover_throttle_holds_altitude() {
        let mut drone = DroneState::new();
        drone.spawn([0.0; 3], 0.0);
        drone.arm();
        let hover = drone.physics.get_hover_throttle(&drone.build, &drone.physics_config);
        // The ESC adds idle under the stick: command what gives hover at the motors.
        drone.throttle = (hover - ARMED_IDLE) / (1.0 - ARMED_IDLE);
        drone.physics.reset(hover, &drone.build, &drone.physics_config);
        for _ in 0..40 {
            drone.physics_tick(0.05, free);
        }
        let y = drone.position[1];
        assert!(y.abs() < 1.5, "hover drifted {y} m in 2 s");
    }

    #[test]
    fn gamepad_throttle_maps_stick_to_throttle() {
        let mut drone = DroneState::new();
        drone.spawn([0.0; 3], 0.0);
        let at = |drone: &mut DroneState, throttle: f32| {
            let input = FrameInput {
                sticks: Some(Sticks { throttle, ..Default::default() }),
                ..Default::default()
            };
            drone.frame(0.016, &input);
            drone.throttle
        };
        assert_eq!(at(&mut drone, -1.0), 0.0);
        assert_eq!(at(&mut drone, 1.0), 1.0);
        // Centre is the stick throttle the ESCs turn into hover once idle is added.
        let centre = at(&mut drone, 0.0);
        let out = ARMED_IDLE + (1.0 - ARMED_IDLE) * centre;
        assert!((out - drone.physics.hover()).abs() < 1e-5, "centre should hover, gives {out}");
    }

    fn pad(roll: f32, pitch: f32) -> FrameInput {
        FrameInput { sticks: Some(Sticks { roll, pitch, ..Default::default() }), ..Default::default() }
    }

    fn flying(mode: FlightMode) -> DroneState {
        let mut drone = DroneState::new();
        drone.spawn([0.0; 3], 0.0);
        drone.controller_config.flight_mode = mode;
        drone.throttle = 0.0;
        assert!(drone.arm());
        drone
    }

    #[test]
    fn the_mode_switch_steps_through_every_mode_and_back() {
        let mut m = FlightMode::Acro;
        let mut seen = vec![m];
        for _ in 0..FlightMode::ALL.len() {
            m = m.next();
            seen.push(m);
        }
        assert_eq!(seen, [FlightMode::Acro, FlightMode::Angle, FlightMode::Horizon, FlightMode::Acro3d, FlightMode::Acro]);
    }

    #[test]
    fn stick_signs_match_the_attitude() {
        // Right roll and forward pitch, in acro, read back as positive roll and pitch.
        let mut drone = flying(FlightMode::Acro);
        drone.frame(0.05, &pad(0.5, 0.0));
        assert!(drone.attitude_deg().0 > 1.0, "{:?}", drone.attitude_deg());
        let mut drone = flying(FlightMode::Acro);
        drone.frame(0.05, &pad(0.0, 0.5));
        assert!(drone.attitude_deg().1 > 1.0, "{:?}", drone.attitude_deg());
    }

    #[test]
    fn angle_mode_holds_the_stick_angle_and_levels_on_release() {
        let mut drone = flying(FlightMode::Angle);
        let limit = drone.controller_config.angle_limit;
        for _ in 0..120 {
            drone.frame(1.0 / 60.0, &pad(1.0, -0.5));
        }
        let (roll, pitch) = drone.attitude_deg();
        assert!((roll - limit).abs() < 3.0, "full right roll tilts to the limit, got {roll}");
        assert!((pitch + limit / 2.0).abs() < 3.0, "half back pitch tilts half way, got {pitch}");
        for _ in 0..120 {
            drone.frame(1.0 / 60.0, &pad(0.0, 0.0));
        }
        assert!(drone.tilt_deg() < 2.0, "centred sticks level it, tilt {}", drone.tilt_deg());
    }

    #[test]
    fn angle_mode_rights_an_upside_down_drone() {
        let mut drone = flying(FlightMode::Angle);
        drone.orientation = drone.orientation.rotate_z(PI * 0.9);
        for _ in 0..180 {
            drone.frame(1.0 / 60.0, &pad(0.0, 0.0));
        }
        assert!(drone.tilt_deg() < 2.0, "tilt {}", drone.tilt_deg());
    }

    #[test]
    fn acro_holds_its_attitude_with_centred_sticks() {
        let mut drone = flying(FlightMode::Acro);
        drone.orientation = drone.orientation.rotate_z(0.5);
        let before = drone.tilt_deg();
        for _ in 0..60 {
            drone.frame(1.0 / 60.0, &pad(0.0, 0.0));
        }
        assert!((drone.tilt_deg() - before).abs() < 1e-3);
    }

    #[test]
    fn horizon_levels_near_centre_and_flips_at_full_stick() {
        let mut drone = flying(FlightMode::Horizon);
        drone.orientation = drone.orientation.rotate_z(0.6);
        for _ in 0..120 {
            drone.frame(1.0 / 60.0, &pad(0.0, 0.0));
        }
        assert!(drone.tilt_deg() < 2.0, "centred: levels, tilt {}", drone.tilt_deg());
        // Full stick for long enough to go right over: horizon lets it roll through.
        let mut most = 0.0f32;
        for _ in 0..60 {
            drone.frame(1.0 / 60.0, &pad(1.0, 0.0));
            most = most.max(drone.tilt_deg());
        }
        assert!(most > 170.0, "full stick rolls past upside down, most tilt {most}");
    }

    #[test]
    fn three_d_mode_reverses_the_throttle() {
        let mut drone = flying(FlightMode::Acro3d);
        let input = FrameInput { sticks: Some(Sticks { throttle: -0.8, ..Default::default() }), ..Default::default() };
        drone.frame(0.016, &input);
        assert!(drone.throttle < -0.7, "the low stick thrusts backwards, {}", drone.throttle);
        let mut drone = flying(FlightMode::Angle);
        drone.frame(0.016, &input);
        assert!(drone.throttle >= 0.0, "other modes never reverse, {}", drone.throttle);
    }

    #[test]
    fn position_is_interpolated_between_ticks() {
        let mut drone = DroneState::new();
        drone.spawn([0.0; 3], 0.0);
        drone.prev_position = [0.0; 3];
        drone.position = [2.0, 0.0, 0.0];
        assert_eq!(drone.position_at(0.5), [1.0, 0.0, 0.0]);
    }
}
