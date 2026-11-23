// The drone in the world: spawned on the ground, armed, flown, crashed and reset.
// The flight model is based on DroneState.java and DroneClientEvents.java from Minecraft FPV (GPL-3.0, github.com/minecraft-fpv/mfpv-client).
// FPV Horizon flies it through a simulated flight controller: the sticks set rates, a rate PID
// drives each motor, and the motors' thrust and torque turn the drone.

use super::build::{BuildParam, DroneBuildConfig};
use super::collision;
use super::controller::{ControlAxis, ControllerConfig, FlightMode};
use super::flight_controller::{self, PidConfig, RateController};
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
/// The flight controller's loop and the airframe's step, seconds (500 Hz).
pub const SIM_DT: f32 = 0.002;
/// The fastest self-levelling turns the drone, deg/s.
pub const LEVEL_MAX_RATE: f32 = 720.0;
/// The mouse's turn rate is smoothed over this long, seconds: frames come unevenly.
pub const MOUSE_RATE_SMOOTHING: f32 = 0.02;

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
    /// How fast it turns, rad/s about its own axes (x right, y up, z back).
    pub angular_velocity: Vec3f,
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
    pub pid: PidConfig,
    pub rate_controller: RateController,
    /// What the pilot asks for, set each frame and flown each loop.
    pub command: Command,
    /// What the ESCs give each motor, -1..1.
    pub motor_output: [f32; 4],
    /// The mouse's smoothed turn rate, deg/s (roll, pitch).
    mouse_rate: [f32; 2],
    /// Time not yet simulated, under a loop.
    sim_time: f32,
}

/// The pilot's command to the flight controller.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Command {
    /// Rates, deg/s, roll pitch and yaw, signed as the sticks turn the drone.
    pub rates: [f32; 3],
    /// The roll and pitch sticks (-1..1), which angle and horizon modes read as a tilt.
    pub level_sticks: (f32, f32),
}

impl Default for DroneState {
    fn default() -> Self {
        Self {
            active: false,
            armed: false,
            crashed: false,
            grounded: false,
            orientation: Quat::IDENTITY,
            angular_velocity: Vec3f::ZERO,
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
            pid: PidConfig::default(),
            rate_controller: RateController::default(),
            command: Command::default(),
            motor_output: [0.0; 4],
            mouse_rate: [0.0; 2],
            sim_time: 0.0,
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
        self.angular_velocity = Vec3f::ZERO;
        self.velocity = Vec3f::ZERO;
        self.throttle = 0.0;
        self.rate_controller.reset();
        self.command = Command::default();
        self.motor_output = [0.0; 4];
        self.mouse_rate = [0.0; 2];
        self.sim_time = 0.0;
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
        self.angular_velocity = Vec3f::ZERO;
    }

    /// Arms if the throttle is at idle and it isn't crashed. Returns whether it is armed.
    pub fn arm(&mut self) -> bool {
        let idle = if self.is_3d() { self.throttle.abs() } else { self.throttle };
        if self.active && !self.crashed && idle <= ARM_THROTTLE && !self.armed {
            self.armed = true;
            self.rate_controller.reset();
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

    /// One rendered frame: read the sticks into rates and the throttle, then fly the flight
    /// controller and the airframe through the frame's time. Runs every frame so the camera is
    /// as smooth as the frame rate; the world moves the drone per tick.
    pub fn frame(&mut self, dt: f32, input: &FrameInput) {
        if !self.active {
            return;
        }
        let dt = dt.clamp(0.0, 0.1);

        self.hud_mouse_roll += ((input.mouse_dx / HUD_MOUSE_FULL_DEG).clamp(-1.0, 1.0) - self.hud_mouse_roll) * HUD_MOUSE_SMOOTHING;
        self.hud_mouse_pitch += ((input.mouse_dy / HUD_MOUSE_FULL_DEG).clamp(-1.0, 1.0) - self.hud_mouse_pitch) * HUD_MOUSE_SMOOTHING;
        if dt > 0.0 {
            // The mouse's turn this frame, as a rate.
            let k = 1.0 - (-dt / MOUSE_RATE_SMOOTHING).exp();
            self.mouse_rate[0] += (input.mouse_dx * ROLL_PER_MOUSE_DEG / dt - self.mouse_rate[0]) * k;
            self.mouse_rate[1] += (input.mouse_dy * PITCH_PER_MOUSE_DEG / dt - self.mouse_rate[1]) * k;
        }

        let mut hud_yaw = input.key_yaw;
        let mut hud_roll = self.hud_mouse_roll;
        let mut hud_pitch = self.hud_mouse_pitch;
        let mut rates = [self.mouse_rate[0], self.mouse_rate[1], input.key_yaw * YAW_RATE_DEG_PER_SEC];
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
            rates[0] += cfg.rate_deg_per_sec(ControlAxis::Roll, roll);
            rates[1] += cfg.rate_deg_per_sec(ControlAxis::Pitch, pitch);
            rates[2] += cfg.rate_deg_per_sec(ControlAxis::Yaw, cfg.shape_stick(sticks.yaw));
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
        self.command = Command { rates, level_sticks };

        self.simulate(dt);

        self.hud_yaw = hud_yaw.clamp(-1.0, 1.0);
        self.hud_roll = hud_roll.clamp(-1.0, 1.0);
        self.hud_pitch = hud_pitch.clamp(-1.0, 1.0);
        self.hud_throttle = self.throttle_to_hud(self.throttle).clamp(-1.0, 1.0);
    }

    /// Runs the flight controller and the airframe for `dt` seconds, in fixed loops.
    pub fn simulate(&mut self, dt: f32) {
        self.sim_time = (self.sim_time + dt.max(0.0)).min(0.25);
        while self.sim_time >= SIM_DT {
            self.sim_time -= SIM_DT;
            self.sim_step(SIM_DT);
        }
    }

    /// One loop: the gyro against the setpoint through the rate PID, the motor mix out to the
    /// ESCs, and the airframe a step on.
    fn sim_step(&mut self, h: f32) {
        let esc = if self.armed {
            let setpoint = self.rate_setpoint();
            let gyro = self.gyro();
            let mut pid = self.rate_controller.update(&self.pid, setpoint, gyro, self.throttle.abs(), h);
            let scale = flight_controller::authority_scale(self.physics.authority());
            pid[0] *= scale[0];
            pid[1] *= scale[1];
            let mix = flight_controller::mix(self.throttle, pid, self.is_3d());
            // The ESCs keep the motors at idle and above while armed.
            mix.map(|o| o.signum() * (ARMED_IDLE + (1.0 - ARMED_IDLE) * o.abs()))
        } else {
            self.rate_controller.reset();
            [0.0; 4]
        };
        self.motor_output = esc;
        self.physics.step_airframe(h, esc, &mut self.orientation, &mut self.velocity, &mut self.angular_velocity, &self.build, &self.physics_config);
    }

    /// The gyro: how fast the drone turns, deg/s of roll, pitch and yaw, signed as the sticks.
    pub fn gyro(&self) -> [f32; 3] {
        let w = self.angular_velocity;
        [-w.z.to_degrees(), -w.x.to_degrees(), -w.y.to_degrees()]
    }

    /// The rates the flight controller aims for now, deg/s: the sticks' in acro, the
    /// self-levelling's in angle, a blend in horizon.
    pub fn rate_setpoint(&self) -> [f32; 3] {
        let mut sp = self.command.rates;
        let sticks = self.command.level_sticks;
        match self.flight_mode() {
            FlightMode::Acro | FlightMode::Acro3d => {}
            FlightMode::Angle => {
                // The sticks set the tilt, not a rate: their roll and pitch rate is replaced.
                let (roll, pitch) = self.level_rate(sticks);
                sp[0] = roll;
                sp[1] = pitch;
            }
            FlightMode::Horizon => {
                // Acro's rates plus levelling that fades out as the sticks go to full, and past
                // a steep tilt, so a full stick flips.
                let deflection = sticks.0.abs().max(sticks.1.abs()).min(1.0);
                let past = ((self.tilt_deg() - HORIZON_LEVEL_TILT_LIMIT) / (180.0 - HORIZON_LEVEL_TILT_LIMIT)).clamp(0.0, 1.0);
                let weight = (1.0 - deflection) * (1.0 - past);
                let (roll, pitch) = self.level_rate(sticks);
                sp[0] += roll * weight;
                sp[1] += pitch * weight;
            }
        }
        sp
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

    /// The self-levelling rates, deg/s of (roll, pitch): toward the tilt the sticks ask for
    /// (`sticks` × the angle limit). As Betaflight, the angle error becomes a rate,
    /// error × strength / 10 per second, which the rate loop then flies.
    fn level_rate(&self, sticks: (f32, f32)) -> (f32, f32) {
        let cfg = &self.controller_config;
        let (roll, pitch) = self.attitude_deg();
        let target = (sticks.0.clamp(-1.0, 1.0) * cfg.angle_limit, sticks.1.clamp(-1.0, 1.0) * cfg.angle_limit);
        let k = cfg.level_strength / 10.0;
        let wrap = |a: f32| (a + 540.0).rem_euclid(360.0) - 180.0;
        let rate = |err: f32| (wrap(err) * k).clamp(-LEVEL_MAX_RATE, LEVEL_MAX_RATE);
        (rate(target.0 - roll), rate(target.1 - pitch))
    }

    /// One world tick of `dt` seconds: the drone moves at its velocity (which the flight loops
    /// set) as far as the world lets it. `mover` gets the drone's position and a movement and
    /// returns the part of it the world allows.
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
            // A knock takes some of the spin out of it.
            self.angular_velocity = self.angular_velocity.mul(0.5);
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
            self.angular_velocity = Vec3f::ZERO;
        } else if self.grounded {
            // Armed on the ground: what the I term would hold against the ground bleeds away.
            self.rate_controller.settle(0.5);
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
            drone.frame(0.05, &FrameInput::default());
            drone.physics_tick(0.05, free);
        }
        let y = drone.position[1];
        assert!(y.abs() < 1.5, "hover drifted {y} m in 2 s");
        assert!(drone.tilt_deg() < 1.0, "and stays level, tilt {}", drone.tilt_deg());
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

    #[test]
    fn the_mix_turns_the_drone_the_way_its_axis_means() {
        // A positive roll, pitch or yaw correction through the mixer and the motors turns the
        // drone right, nose down or right, and not much on the other axes.
        for axis in 0..3 {
            let mut drone = DroneState::new();
            drone.spawn([0.0; 3], 0.0);
            let hover = drone.physics.get_hover_throttle(&drone.build, &drone.physics_config);
            drone.physics.reset(hover, &drone.build, &drone.physics_config);
            let mut pid = [0.0; 3];
            pid[axis] = 0.05;
            let esc = flight_controller::mix(hover, pid, false);
            for _ in 0..10 {
                drone.physics.step_airframe(SIM_DT, esc, &mut drone.orientation, &mut drone.velocity, &mut drone.angular_velocity, &drone.build, &drone.physics_config);
            }
            let gyro = drone.gyro();
            eprintln!("axis {axis}: gyro {gyro:?}");
            assert!(gyro[axis] > 5.0, "axis {axis} turns the right way: {gyro:?}");
            for other in (0..3).filter(|&o| o != axis) {
                assert!(gyro[other].abs() < gyro[axis] * 0.2, "axis {axis} leaks into {other}: {gyro:?}");
            }
        }
    }

    /// Preset `preset` armed and hovering on centred sticks (a gamepad's centre is hover).
    fn hovering(preset: &crate::drone::Preset) -> DroneState {
        let mut drone = DroneState::new();
        drone.build = preset.build();
        drone.spawn([0.0; 3], 0.0);
        assert!(drone.arm());
        fly(&mut drone, 1.0, &pad(0.0, 0.0));
        drone
    }

    /// Flies `seconds` on `input`, a frame per flight-controller loop.
    fn fly(drone: &mut DroneState, seconds: f32, input: &FrameInput) {
        for _ in 0..(seconds / SIM_DT).round() as usize {
            drone.frame(SIM_DT, input);
        }
    }

    fn sticks(roll: f32, pitch: f32, yaw: f32) -> FrameInput {
        FrameInput { sticks: Some(Sticks { roll, pitch, yaw, throttle: 0.0 }), ..Default::default() }
    }

    fn worst_rate(drone: &DroneState) -> f32 {
        drone.gyro().iter().fold(0.0f32, |a, g| a.max(g.abs()))
    }

    /// Time, ms, from a full stick on `axis` until the gyro reaches 90% of the rate it asks for.
    fn rise_time(drone: &mut DroneState, axis: usize) -> f32 {
        let mut s = [0.0; 3];
        s[axis] = 1.0;
        let input = sticks(s[0], s[1], s[2]);
        let control = [ControlAxis::Roll, ControlAxis::Pitch, ControlAxis::Yaw][axis];
        let target = drone.controller_config.rate_deg_per_sec(control, 1.0);
        for i in 1..=200 {
            drone.frame(SIM_DT, &input);
            if drone.gyro()[axis] >= 0.9 * target {
                return i as f32 * SIM_DT * 1000.0;
            }
        }
        f32::INFINITY
    }

    #[test]
    fn rates_reach_the_betaflight_target_within_about_50_ms() {
        for preset in crate::drone::PRESETS.iter() {
            for axis in 0..3 {
                let mut drone = hovering(preset);
                let t = rise_time(&mut drone, axis);
                eprintln!("{} axis {axis}: 90% of full rate in {t:.0} ms", preset.id);
                // Roll and pitch in about 50 ms (a 7" is heavier); yaw, turned only by prop
                // drag, slower.
                let limit = if axis == 2 { 150.0 } else { 60.0 };
                assert!(t <= limit, "{} axis {axis}: {t} ms", preset.id);
                // Not instant either: the motors and the frame's inertia take their time.
                assert!(t >= 10.0, "{} axis {axis}: {t} ms is a perfect rotation", preset.id);
            }
        }
    }

    #[test]
    fn a_flip_overshoots_a_little_and_settles() {
        for preset in crate::drone::PRESETS.iter() {
            let mut drone = hovering(preset);
            let target = drone.controller_config.rate_deg_per_sec(ControlAxis::Roll, 1.0);
            // Throttle cut and full right roll until it is most of the way round, as a pilot
            // lets go early.
            let flip = FrameInput { sticks: Some(Sticks { roll: 1.0, throttle: -1.0, ..Default::default() }), ..Default::default() };
            let (mut turned, mut peak) = (0.0, 0.0f32);
            while turned < 300.0 {
                drone.frame(SIM_DT, &flip);
                turned += drone.gyro()[0] * SIM_DT;
                peak = peak.max(drone.gyro()[0]);
            }
            // Let go, and punch out to catch it, as a pilot does so it doesn't sink into its own
            // prop wash.
            let catch = FrameInput { sticks: Some(Sticks { throttle: 1.0, ..Default::default() }), ..Default::default() };
            let mut bounce = 0.0f32;
            for _ in 0..100 {
                drone.frame(SIM_DT, &catch);
                bounce = bounce.min(drone.gyro()[0]);
            }
            let settled = worst_rate(&drone);
            fly(&mut drone, 0.3, &catch);
            let held = worst_rate(&drone);
            eprintln!("{}: peak {peak:.0}/{target:.0} deg/s, bounce back {bounce:.1}, 200 ms after {settled:.1}, then {held:.2}", preset.id);
            // A tiny whoop's motors idle far below a hover and spool up slowly, so it brakes
            // late, and the punch out drags it through its own wash: it shakes for longer, as
            // the real ones do.
            let tiny = preset.mass < 0.1;
            let (most, settle, hold) = if tiny { (0.3, 120.0, 5.0) } else { (0.15, 5.0, 1.0) };
            let over = peak / target - 1.0;
            assert!(over > 0.002 && over < most, "{}: the rate overshoots a little, {:.1}%", preset.id, over * 100.0);
            assert!(bounce < -0.5 && bounce > -most * target, "{}: it stops with a small bounce back, {bounce}", preset.id);
            assert!(settled < settle, "{}: settled 200 ms after the flip, still {settled} deg/s", preset.id);
            assert!(held < hold, "{}: and holds, {held} deg/s", preset.id);
        }
    }

    #[test]
    fn hover_is_stable_and_shrugs_off_a_knock() {
        for preset in crate::drone::PRESETS.iter() {
            let mut drone = hovering(preset);
            let mut worst = 0.0f32;
            for _ in 0..1000 {
                drone.frame(SIM_DT, &sticks(0.0, 0.0, 0.0));
                worst = worst.max(worst_rate(&drone));
            }
            eprintln!("{}: hover worst {worst:.2} deg/s, tilt {:.2}", preset.id, drone.tilt_deg());
            assert!(worst < 2.0, "{}: steady hover, worst {worst} deg/s", preset.id);
            assert!(drone.tilt_deg() < 1.0, "{}: level, tilt {}", preset.id, drone.tilt_deg());
            // A knock: 200 deg/s of roll and pitch. The loop stops it within 50 ms, then the
            // I term, remembering the attitude it held, eases it back.
            drone.angular_velocity = Vec3f::new(-200f32.to_radians(), 0.0, 200f32.to_radians());
            fly(&mut drone, 0.05, &sticks(0.0, 0.0, 0.0));
            let stopped = worst_rate(&drone);
            fly(&mut drone, 0.35, &sticks(0.0, 0.0, 0.0));
            let left = worst_rate(&drone);
            eprintln!("{}: knock 200 deg/s, {stopped:.0} after 50 ms, {left:.1} after 400 ms", preset.id);
            assert!(stopped < 80.0, "{}: knock stopped, {stopped} deg/s after 50 ms", preset.id);
            assert!(left < 3.0, "{}: knock damped in 400 ms, {left} deg/s left", preset.id);
            assert!(drone.tilt_deg() < 3.0, "{}: and back level, tilt {}", preset.id, drone.tilt_deg());
        }
    }

    #[test]
    fn the_motors_spin_apart_to_turn_it() {
        let mut drone = hovering(&crate::drone::PRESETS[0]);
        let even = drone.physics.motor_vel.map(f32::abs);
        assert!(even.iter().all(|v| (v - even[0]).abs() < even[0] * 0.02), "hover: every motor alike, {even:?}");
        fly(&mut drone, 0.02, &sticks(1.0, 0.0, 0.0));
        let speed = drone.physics.motor_vel.map(f32::abs);
        // Right roll: the left motors (2, 3) faster than the right ones.
        assert!(speed[2].min(speed[3]) > speed[0].max(speed[1]) * 1.2, "{speed:?}");
    }

    #[test]
    fn sinking_into_its_own_wash_shakes_it() {
        let mut drone = hovering(&crate::drone::PRESETS[0]);
        assert!(drone.physics.prop_wash < 0.01);
        // Dropping straight down through its own downwash at hover throttle.
        drone.velocity = Vec3f::new(0.0, -8.0, 0.0);
        let (mut worst, mut wash) = (0.0f32, 0.0f32);
        for _ in 0..150 {
            drone.frame(SIM_DT, &sticks(0.0, 0.0, 0.0));
            wash = wash.max(drone.physics.prop_wash);
            worst = drone.gyro()[..2].iter().fold(worst, |a, g| a.max(g.abs()));
        }
        eprintln!("wash {wash:.2}, worst {worst:.1} deg/s, tilt {:.1}", drone.tilt_deg());
        assert!(wash > 0.3, "deep in the wash, {wash}");
        assert!(worst > 3.0, "the wash rocks it, {worst} deg/s");
        assert!(drone.tilt_deg() < 30.0, "but the loop keeps it upright, tilt {}", drone.tilt_deg());
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
