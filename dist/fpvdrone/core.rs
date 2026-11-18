// Game-agnostic core logic for the FPV Drone mod.
// Handles 3D vector/quaternion math, aerodynamics, motor physics, collision response,
// Betaflight rates, stick shaping, HUD geometry, and drone state lifecycle.

pub mod build {
    // Drone hardware specification and physical dimensions.
    // Direct port of DroneBuild.java.

    use std::f32::consts::PI;

    pub const MOTOR_DENSITY: f32 = 13240.0;
    pub const BELL_MASS_FRACTION: f32 = 0.36363637;
    pub const BLADE_DENSITY: f32 = 1200.0;
    pub const BLADE_THICKNESS: f32 = 0.0013;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BuildParam {
        DroneMass,
        BatteryCells,
        MotorKv,
        Blades,
        PropRadius,
        PropPitch,
        PropWidth,
        MotorWidth,
        MotorHeight,
        CameraAngle,
    }

    impl BuildParam {
        pub const ALL: [BuildParam; 10] = [
            BuildParam::DroneMass,
            BuildParam::BatteryCells,
            BuildParam::MotorKv,
            BuildParam::Blades,
            BuildParam::PropRadius,
            BuildParam::PropPitch,
            BuildParam::PropWidth,
            BuildParam::MotorWidth,
            BuildParam::MotorHeight,
            BuildParam::CameraAngle,
        ];

        pub fn key(self) -> &'static str {
            match self {
                BuildParam::DroneMass => "drone_mass",
                BuildParam::BatteryCells => "battery_cells",
                BuildParam::MotorKv => "motor_kv",
                BuildParam::Blades => "blades",
                BuildParam::PropRadius => "prop_radius",
                BuildParam::PropPitch => "prop_pitch",
                BuildParam::PropWidth => "prop_width",
                BuildParam::MotorWidth => "motor_width",
                BuildParam::MotorHeight => "motor_height",
                BuildParam::CameraAngle => "camera_angle",
            }
        }

        pub fn default_value(self) -> f32 {
            match self {
                BuildParam::DroneMass => 0.65,
                BuildParam::BatteryCells => 2.0,
                BuildParam::MotorKv => 2000.0,
                BuildParam::Blades => 3.0,
                BuildParam::PropRadius => 0.0635,
                BuildParam::PropPitch => 0.1,
                BuildParam::PropWidth => 0.015,
                BuildParam::MotorWidth => 0.023,
                BuildParam::MotorHeight => 0.006,
                BuildParam::CameraAngle => 10.0,
            }
        }

        pub fn min(self) -> f32 {
            match self {
                BuildParam::DroneMass => 0.1,
                BuildParam::BatteryCells => 1.0,
                BuildParam::MotorKv => 500.0,
                BuildParam::Blades => 2.0,
                BuildParam::PropRadius => 0.025,
                BuildParam::PropPitch => 0.03,
                BuildParam::PropWidth => 0.005,
                BuildParam::MotorWidth => 0.01,
                BuildParam::MotorHeight => 0.003,
                BuildParam::CameraAngle => 0.0,
            }
        }

        pub fn max(self) -> f32 {
            match self {
                BuildParam::DroneMass => 2.0,
                BuildParam::BatteryCells => 8.0,
                BuildParam::MotorKv => 5000.0,
                BuildParam::Blades => 6.0,
                BuildParam::PropRadius => 0.15,
                BuildParam::PropPitch => 0.25,
                BuildParam::PropWidth => 0.04,
                BuildParam::MotorWidth => 0.06,
                BuildParam::MotorHeight => 0.03,
                BuildParam::CameraAngle => 70.0,
            }
        }

        pub fn step(self) -> f32 {
            match self {
                BuildParam::DroneMass => 0.005,
                BuildParam::BatteryCells => 1.0,
                BuildParam::MotorKv => 50.0,
                BuildParam::Blades => 1.0,
                BuildParam::PropRadius => 0.0005,
                BuildParam::PropPitch => 0.001,
                BuildParam::PropWidth => 0.0005,
                BuildParam::MotorWidth => 0.0005,
                BuildParam::MotorHeight => 0.0005,
                BuildParam::CameraAngle => 1.0,
            }
        }

        pub fn snap(self, val: f32) -> f32 {
            let min = self.min();
            let max = self.max();
            let step = self.step();
            let steps = ((val - min) / step).round();
            (min + steps * step).clamp(min, max)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct DroneBuildConfig {
        pub values: [f32; 10],
        pub version: u32,
    }

    impl Default for DroneBuildConfig {
        fn default() -> Self {
            let mut vals = [0.0; 10];
            for (i, p) in BuildParam::ALL.iter().enumerate() {
                vals[i] = p.default_value();
            }
            Self { values: vals, version: 1 }
        }
    }

    impl DroneBuildConfig {
        pub fn get(&self, param: BuildParam) -> f32 {
            self.values[param as usize]
        }

        pub fn set(&mut self, param: BuildParam, value: f32) {
            self.values[param as usize] = param.snap(value);
            self.version = self.version.wrapping_add(1);
        }

        pub fn reset(&mut self) {
            *self = Self::default();
            self.version = self.version.wrapping_add(1);
        }

        pub fn motor_mass(&self) -> f32 {
            let radius = self.get(BuildParam::MotorWidth) * 0.5;
            let volume = PI * radius * radius * self.get(BuildParam::MotorHeight);
            MOTOR_DENSITY * volume
        }

        pub fn bell_mass(&self) -> f32 {
            BELL_MASS_FRACTION * self.motor_mass()
        }

        pub fn blade_mass(&self) -> f32 {
            let volume = self.get(BuildParam::PropRadius) * self.get(BuildParam::PropWidth) * BLADE_THICKNESS;
            BLADE_DENSITY * volume
        }

        pub fn parse_properties(&mut self, text: &str) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    for param in BuildParam::ALL {
                        if param.key() == k {
                            if let Ok(num) = v.parse::<f32>() {
                                self.set(param, num);
                            }
                        }
                    }
                }
            }
        }

        pub fn to_properties(&self) -> String {
            let mut out = String::from("# FPV Drone build config. SI units: kg, m, rpm/V.\n");
            for param in BuildParam::ALL {
                out.push_str(&format!("{}={}\n", param.key(), self.get(param)));
            }
            out
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_build_defaults_and_masses() {
            let cfg = DroneBuildConfig::default();
            assert_eq!(cfg.get(BuildParam::DroneMass), 0.65);
            assert_eq!(cfg.get(BuildParam::BatteryCells), 2.0);
            let m_mass = cfg.motor_mass();
            assert!(m_mass > 0.01 && m_mass < 0.1);
            let b_mass = cfg.blade_mass();
            assert!(b_mass > 0.0005 && b_mass < 0.01);
        }

        #[test]
        fn test_properties_round_trip() {
            let mut cfg = DroneBuildConfig::default();
            cfg.set(BuildParam::BatteryCells, 4.0);
            let prop = cfg.to_properties();
            let mut loaded = DroneBuildConfig::default();
            loaded.parse_properties(&prop);
            assert_eq!(loaded.get(BuildParam::BatteryCells), 4.0);
            assert!((loaded.get(BuildParam::DroneMass) - 0.65).abs() < 1e-4);
        }
    }
}
pub mod collision {
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
}
pub mod controller {
    // Betaflight rate model, stick shaping, and flight control inputs.
    // Port of DroneController.java and VirtualJoystick.java.

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ControlAxis {
        Roll = 0,
        Pitch = 1,
        Yaw = 2,
        Throttle = 3,
    }

    impl ControlAxis {
        pub const ALL: [ControlAxis; 4] = [
            ControlAxis::Roll,
            ControlAxis::Pitch,
            ControlAxis::Yaw,
            ControlAxis::Throttle,
        ];

        pub fn key(self) -> &'static str {
            match self {
                ControlAxis::Roll => "roll",
                ControlAxis::Pitch => "pitch",
                ControlAxis::Yaw => "yaw",
                ControlAxis::Throttle => "throttle",
            }
        }
    }

    pub const RC_RATE_MIN: f32 = 0.01;
    pub const RC_RATE_MAX: f32 = 2.55;
    pub const SUPER_RATE_MIN: f32 = 0.0;
    pub const SUPER_RATE_MAX: f32 = 0.99;
    pub const EXPO_MIN: f32 = 0.0;
    pub const EXPO_MAX: f32 = 1.0;

    pub const DEFAULT_RC_RATE: f32 = 0.67;
    pub const DEFAULT_SUPER_RATE: f32 = 0.2;
    pub const DEFAULT_EXPO: f32 = 0.67;
    pub const DEFAULT_DEADZONE: f32 = 0.05;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ControllerConfig {
        pub deadzone: f32,
        pub rc_rate: [f32; 4],
        pub super_rate: [f32; 4],
        pub expo: [f32; 4],
        pub mode_3d: bool,
        pub show_axis_values: bool,
        pub show_head_debug: bool,
    }

    impl Default for ControllerConfig {
        fn default() -> Self {
            Self {
                deadzone: DEFAULT_DEADZONE,
                rc_rate: [DEFAULT_RC_RATE; 4],
                super_rate: [DEFAULT_SUPER_RATE; 4],
                expo: [DEFAULT_EXPO; 4],
                mode_3d: false,
                show_axis_values: false,
                show_head_debug: false,
            }
        }
    }

    impl ControllerConfig {
        pub fn rate_deg_per_sec(&self, axis: ControlAxis, stick: f32) -> f32 {
            let i = axis as usize;
            let abs = stick.abs();
            let shaped = stick * abs * abs * self.expo[i] + stick * (1.0 - self.expo[i]);
            let mut rate = self.rc_rate[i];
            if rate > 2.0 {
                rate += 14.54 * (rate - 2.0);
            }
            let mut deg_per_sec = 200.0 * rate * shaped;
            if self.super_rate[i] > 0.0 {
                let factor = 1.0 / (1.0 - abs * self.super_rate[i]).clamp(0.01, 1.0);
                deg_per_sec *= factor;
            }
            deg_per_sec
        }

        pub fn shape_stick(&self, v: f32) -> f32 {
            let a = v.abs();
            if a < self.deadzone {
                return 0.0;
            }
            let scaled = (a - self.deadzone) / (1.0 - self.deadzone);
            scaled.min(1.0).copysign(v)
        }

        pub fn parse_properties(&mut self, text: &str) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    if k == "deadzone" {
                        if let Ok(num) = v.parse::<f32>() { self.deadzone = num; }
                    } else if k == "mode_3d" {
                        self.mode_3d = v.eq_ignore_ascii_case("true");
                    } else if k == "show_axis_values" {
                        self.show_axis_values = v.eq_ignore_ascii_case("true");
                    } else if k == "show_head_debug" {
                        self.show_head_debug = v.eq_ignore_ascii_case("true");
                    } else {
                        for a in ControlAxis::ALL {
                            let i = a as usize;
                            if k == format!("{}_rc_rate", a.key()) {
                                if let Ok(num) = v.parse::<f32>() { self.rc_rate[i] = num.clamp(RC_RATE_MIN, RC_RATE_MAX); }
                            } else if k == format!("{}_super_rate", a.key()) {
                                if let Ok(num) = v.parse::<f32>() { self.super_rate[i] = num.clamp(SUPER_RATE_MIN, SUPER_RATE_MAX); }
                            } else if k == format!("{}_expo", a.key()) {
                                if let Ok(num) = v.parse::<f32>() { self.expo[i] = num.clamp(EXPO_MIN, EXPO_MAX); }
                            }
                        }
                    }
                }
            }
        }

        pub fn to_properties(&self) -> String {
            let mut out = String::from("# FPV Drone controller config\n");
            out.push_str(&format!("deadzone={}\n", self.deadzone));
            out.push_str(&format!("mode_3d={}\n", self.mode_3d));
            out.push_str(&format!("show_axis_values={}\n", self.show_axis_values));
            out.push_str(&format!("show_head_debug={}\n", self.show_head_debug));
            for a in ControlAxis::ALL {
                let i = a as usize;
                out.push_str(&format!("{}_rc_rate={}\n", a.key(), self.rc_rate[i]));
                out.push_str(&format!("{}_super_rate={}\n", a.key(), self.super_rate[i]));
                out.push_str(&format!("{}_expo={}\n", a.key(), self.expo[i]));
            }
            out
        }
    }

    /// Keyboard-driven virtual joystick state (WASD + mouse delta).
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct VirtualStick {
        pub roll: f32,
        pub pitch: f32,
        pub throttle: f32,
        pub yaw: f32,
    }

    impl Default for VirtualStick {
        fn default() -> Self {
            Self {
                roll: 0.0,
                pitch: 0.0,
                throttle: 0.0,
                yaw: 0.0,
            }
        }
    }

    impl VirtualStick {
        pub fn tick_step(&mut self, left_right: f32, down_up: f32, a_d: f32, s_w: f32, mode_3d: bool) {
            self.roll = Self::approach(self.roll, left_right);
            self.pitch = Self::approach(self.pitch, down_up);
            self.yaw = Self::approach(self.yaw, a_d);

            let min_throttle = if mode_3d { -1.0 } else { 0.0 };
            self.throttle = (self.throttle + s_w * 0.04).clamp(min_throttle, 1.0);
        }

        fn approach(curr: f32, target: f32) -> f32 {
            let step = if target == 0.0 { 0.5 } else { 0.25 };
            let d = target - curr;
            curr + d.clamp(-step, step)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_rates_calculation() {
            let cfg = ControllerConfig::default();
            let deg_full = cfg.rate_deg_per_sec(ControlAxis::Roll, 1.0);
            let deg_half = cfg.rate_deg_per_sec(ControlAxis::Roll, 0.5);
            assert!(deg_full > deg_half);
            assert!(deg_full > 100.0);
        }

        #[test]
        fn test_stick_shaping_deadzone() {
            let cfg = ControllerConfig::default();
            assert_eq!(cfg.shape_stick(0.02), 0.0);
            assert!(cfg.shape_stick(0.1) > 0.0);
        }
    }
}
pub mod hud {
    // FPV Drone HUD gimbal overlays (Betaflight style stick displays).
    // Port of DroneHud.java.

    pub const BOX_SIZE: i32 = 56;
    pub const MARGIN_X: i32 = 14;
    pub const MARGIN_Y: i32 = 14;
    pub const MARKER_HALF: i32 = 3;
    pub const COLOR_OVERLAY: u32 = 0x99FFFFFF; // ARGB semi-transparent white (-1711276033)

    pub trait HudPainter {
        /// Fills an axis-aligned rectangle [min_x, min_y, max_x, max_y] with an ARGB color.
        fn fill_rect(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, argb: u32);
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct GimbalHud {
        pub yaw: f32,
        pub throttle: f32,
        pub roll: f32,
        pub pitch: f32,
    }

    impl GimbalHud {
        pub fn new(yaw: f32, throttle: f32, roll: f32, pitch: f32) -> Self {
            Self {
                yaw: yaw.clamp(-1.0, 1.0),
                throttle: throttle.clamp(-1.0, 1.0),
                roll: roll.clamp(-1.0, 1.0),
                pitch: pitch.clamp(-1.0, 1.0),
            }
        }

        /// Renders both stick gimbals:
        /// - Left gimbal: Yaw (X) & Throttle (Y)
        /// - Right gimbal: Roll (X) & Pitch (Y)
        pub fn render<P: HudPainter>(&self, painter: &mut P, screen_width: i32, screen_height: i32) {
            let top = screen_height - MARGIN_Y - BOX_SIZE;
            // Left gimbal (Throttle / Yaw)
            Self::draw_gimbal(painter, MARGIN_X, top, self.yaw, self.throttle);
            // Right gimbal (Roll / Pitch)
            Self::draw_gimbal(painter, screen_width - MARGIN_X - BOX_SIZE, top, self.roll, self.pitch);
        }

        fn draw_gimbal<P: HudPainter>(painter: &mut P, left: i32, top: i32, x: f32, y: f32) {
            let right = left + BOX_SIZE;
            let bottom = top + BOX_SIZE;
            let cx = left + BOX_SIZE / 2;
            let cy = top + BOX_SIZE / 2;

            // Crosshairs
            painter.fill_rect(left, cy, right, cy + 1, COLOR_OVERLAY);
            painter.fill_rect(cx, top, cx + 1, bottom, COLOR_OVERLAY);

            // Marker box
            let travel = 27.0 - MARKER_HALF as f32;
            let mx = cx + (x.clamp(-1.0, 1.0) * travel).round() as i32;
            let my = cy - (y.clamp(-1.0, 1.0) * travel).round() as i32;

            let l = mx - MARKER_HALF;
            let r = mx + MARKER_HALF + 1;
            let t = my - MARKER_HALF;
            let bt = my + MARKER_HALF + 1;

            // Border of marker
            painter.fill_rect(l, t, r, t + 1, COLOR_OVERLAY);
            painter.fill_rect(l, bt - 1, r, bt, COLOR_OVERLAY);
            painter.fill_rect(l, t + 1, l + 1, bt - 1, COLOR_OVERLAY);
            painter.fill_rect(r - 1, t + 1, r, bt - 1, COLOR_OVERLAY);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct MockPainter {
            rects: Vec<(i32, i32, i32, i32, u32)>,
        }

        impl HudPainter for MockPainter {
            fn fill_rect(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, argb: u32) {
                self.rects.push((min_x, min_y, max_x, max_y, argb));
            }
        }

        #[test]
        fn test_hud_rendering_produces_rectangles() {
            let mut painter = MockPainter { rects: Vec::new() };
            let hud = GimbalHud::new(0.0, 0.5, 0.2, -0.3);
            hud.render(&mut painter, 854, 480);
            assert!(!painter.rects.is_empty());
            // Each gimbal produces 2 crosshair rects + 4 marker border rects = 6 rects * 2 = 12 rects
            assert_eq!(painter.rects.len(), 12);
        }
    }
}
pub mod math {
    // 3D vector and Quaternion math for FPV Drone simulation.
    // Implements vector arithmetic, projections, and quaternion rotations matching JOML / Minecraft conventions.

    use std::f32::consts::PI;

    /// A 3D vector of single-precision floats.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Vec3f {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    impl Vec3f {
        pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
        pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
        pub const DOWN: Self = Self { x: 0.0, y: -1.0, z: 0.0 };
        pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: -1.0 };
        pub const BACK: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
        pub const LEFT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
        pub const RIGHT: Self = Self { x: -1.0, y: 0.0, z: 0.0 };

        pub const fn new(x: f32, y: f32, z: f32) -> Self {
            Self { x, y, z }
        }

        pub fn add(self, o: Self) -> Self {
            Self { x: self.x + o.x, y: self.y + o.y, z: self.z + o.z }
        }

        pub fn sub(self, o: Self) -> Self {
            Self { x: self.x - o.x, y: self.y - o.y, z: self.z - o.z }
        }

        pub fn mul(self, scalar: f32) -> Self {
            Self { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
        }

        pub fn dot(self, o: Self) -> f32 {
            self.x * o.x + self.y * o.y + self.z * o.z
        }

        pub fn cross(self, o: Self) -> Self {
            Self {
                x: self.y * o.z - self.z * o.y,
                y: self.z * o.x - self.x * o.z,
                z: self.x * o.y - self.y * o.x,
            }
        }

        pub fn length_squared(self) -> f32 {
            self.dot(self)
        }

        pub fn length(self) -> f32 {
            self.length_squared().sqrt()
        }

        pub fn normalized_or_zero(self) -> Self {
            let len_sq = self.length_squared();
            if len_sq > 1e-12 {
                let inv = 1.0 / len_sq.sqrt();
                self.mul(inv)
            } else {
                Self::ZERO
            }
        }

        pub fn angle_between(self, o: Self) -> f32 {
            let denom = (self.length_squared() * o.length_squared()).sqrt();
            if denom < 1e-12 {
                return 0.0;
            }
            let dot = (self.dot(o) / denom).clamp(-1.0, 1.0);
            dot.acos()
        }

        pub fn project(self, on: Self) -> Self {
            let on_len_sq = on.length_squared();
            if on_len_sq < 1e-12 {
                Self::ZERO
            } else {
                on.mul(self.dot(on) / on_len_sq)
            }
        }

        pub fn fma(self, scalar: f32, addend: Self) -> Self {
            Self {
                x: self.x + scalar * addend.x,
                y: self.y + scalar * addend.y,
                z: self.z + scalar * addend.z,
            }
        }
    }

    impl std::ops::Add for Vec3f {
        type Output = Self;
        fn add(self, rhs: Self) -> Self { self.add(rhs) }
    }

    impl std::ops::Sub for Vec3f {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self { self.sub(rhs) }
    }

    impl std::ops::Mul<f32> for Vec3f {
        type Output = Self;
        fn mul(self, rhs: f32) -> Self { self.mul(rhs) }
    }

    /// A quaternion (x, y, z, w) representing 3D rotation.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Quat {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }

    impl Quat {
        pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

        pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
            Self { x, y, z, w }
        }

        pub fn from_axis_angle(axis: Vec3f, angle_rad: f32) -> Self {
            let half = angle_rad * 0.5;
            let s = half.sin();
            let norm_axis = axis.normalized_or_zero();
            Self {
                x: norm_axis.x * s,
                y: norm_axis.y * s,
                z: norm_axis.z * s,
                w: half.cos(),
            }
        }

        /// Rotation with Euler order YXZ (Yaw around Y, Pitch around X, Roll around Z).
        /// Matches JOML's `rotationYXZ(angleY, angleX, angleZ)`.
        pub fn rotation_y_x_z(yaw: f32, pitch: f32, roll: f32) -> Self {
            let qy = Self::from_axis_angle(Vec3f::new(0.0, 1.0, 0.0), yaw);
            let qx = Self::from_axis_angle(Vec3f::new(1.0, 0.0, 0.0), pitch);
            let qz = Self::from_axis_angle(Vec3f::new(0.0, 0.0, 1.0), roll);
            qy.mul(qx).mul(qz)
        }

        pub fn mul(self, o: Self) -> Self {
            Self {
                w: self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
                x: self.w * o.x + self.x * o.w + self.y * o.z - self.z * o.y,
                y: self.w * o.y - self.x * o.z + self.y * o.w + self.z * o.x,
                z: self.w * o.z + self.x * o.y - self.y * o.x + self.z * o.w,
            }
        }

        pub fn rotate_x(self, angle_rad: f32) -> Self {
            let q = Self::from_axis_angle(Vec3f::new(1.0, 0.0, 0.0), angle_rad);
            self.mul(q)
        }

        pub fn rotate_y(self, angle_rad: f32) -> Self {
            let q = Self::from_axis_angle(Vec3f::new(0.0, 1.0, 0.0), angle_rad);
            self.mul(q)
        }

        pub fn rotate_z(self, angle_rad: f32) -> Self {
            let q = Self::from_axis_angle(Vec3f::new(0.0, 0.0, 1.0), angle_rad);
            self.mul(q)
        }

        pub fn rotate_axis(self, angle_rad: f32, axis: Vec3f) -> Self {
            let q = Self::from_axis_angle(axis, angle_rad);
            self.mul(q)
        }

        pub fn length_squared(self) -> f32 {
            self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
        }

        pub fn normalize(self) -> Self {
            let len_sq = self.length_squared();
            if len_sq > 1e-12 {
                let inv = 1.0 / len_sq.sqrt();
                Self {
                    x: self.x * inv,
                    y: self.y * inv,
                    z: self.z * inv,
                    w: self.w * inv,
                }
            } else {
                Self::IDENTITY
            }
        }

        /// Transforms a 3D vector by this quaternion rotation.
        pub fn transform_vec3(self, v: Vec3f) -> Vec3f {
            let qv = Vec3f::new(self.x, self.y, self.z);
            let uv = qv.cross(v);
            let uuv = qv.cross(uv);
            v.add((uv.mul(self.w).add(uuv)).mul(2.0))
        }

        /// Spherical linear interpolation between two quaternions.
        pub fn slerp(self, mut target: Self, alpha: f32) -> Self {
            let mut dot = self.x * target.x + self.y * target.y + self.z * target.z + self.w * target.w;
            if dot < 0.0 {
                target.x = -target.x;
                target.y = -target.y;
                target.z = -target.z;
                target.w = -target.w;
                dot = -dot;
            }

            if dot > 0.9995 {
                // Linear interpolation if too close to avoid division by zero
                let x = self.x + alpha * (target.x - self.x);
                let y = self.y + alpha * (target.y - self.y);
                let z = self.z + alpha * (target.z - self.z);
                let w = self.w + alpha * (target.w - self.w);
                return Self::new(x, y, z, w).normalize();
            }

            let theta_0 = dot.clamp(-1.0, 1.0).acos();
            let theta = theta_0 * alpha;
            let sin_theta = theta.sin();
            let sin_theta_0 = theta_0.sin();

            let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
            let s1 = sin_theta / sin_theta_0;

            Self {
                x: s0 * self.x + s1 * target.x,
                y: s0 * self.y + s1 * target.y,
                z: s0 * self.z + s1 * target.z,
                w: s0 * self.w + s1 * target.w,
            }.normalize()
        }

        /// Decomposes quaternion into Tait-Bryan angles with rotation order YXZ.
        /// Returns [yaw, pitch, roll] in radians.
        pub fn to_euler_angles_yxz(self) -> [f32; 3] {
            let (x, y, z, w) = (self.x, self.y, self.z, self.w);
            let sin_p = 2.0 * (w * x - y * z);
            let pitch = if sin_p.abs() >= 1.0 {
                (PI / 2.0).copysign(sin_p)
            } else {
                sin_p.asin()
            };

            let yaw = (2.0 * (w * y + x * z)).atan2(1.0 - 2.0 * (x * x + y * y));
            let roll = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (x * x + z * z));

            [yaw, pitch, roll]
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_vec3_operations() {
            let v1 = Vec3f::new(1.0, 2.0, 3.0);
            let v2 = Vec3f::new(4.0, 5.0, 6.0);
            assert_eq!(v1.add(v2), Vec3f::new(5.0, 7.0, 9.0));
            assert_eq!(v1.dot(v2), 32.0);
            assert_eq!(v1.mul(2.0), Vec3f::new(2.0, 4.0, 6.0));
        }

        #[test]
        fn test_quat_rotation_identity() {
            let q = Quat::IDENTITY;
            let v = Vec3f::new(1.0, 2.0, 3.0);
            let rotated = q.transform_vec3(v);
            assert!((rotated.x - v.x).abs() < 1e-5);
            assert!((rotated.y - v.y).abs() < 1e-5);
            assert!((rotated.z - v.z).abs() < 1e-5);
        }

        #[test]
        fn test_quat_rotation_90_deg_y() {
            let q = Quat::from_axis_angle(Vec3f::UP, PI / 2.0);
            let v = Vec3f::new(0.0, 0.0, -1.0); // Facing forward
            let rotated = q.transform_vec3(v);
            // Rotated 90 degrees around Y: forward (-Z) should become left (-X)
            assert!((rotated.x - (-1.0)).abs() < 1e-5);
            assert!((rotated.z - 0.0).abs() < 1e-5);
        }

        #[test]
        fn test_quat_euler_decomposition() {
            let q = Quat::rotation_y_x_z(0.3, 0.4, 0.5);
            let [yaw, pitch, roll] = q.to_euler_angles_yxz();
            assert!((yaw - 0.3).abs() < 1e-4);
            assert!((pitch - 0.4).abs() < 1e-4);
            assert!((roll - 0.5).abs() < 1e-4);
        }
    }
}
pub mod orientations {
    // Slerp-smoothed orientation tracking for remote drone players.
    // Port of DroneOrientations.java.

    use super::math::Quat;
    use std::collections::HashMap;

    const SMOOTHING_SECONDS: f32 = 0.04;

    #[derive(Debug, Clone)]
    struct OrientationEntry {
        target: Quat,
        display: Quat,
        last_nanos: u64,
    }

    #[derive(Debug, Default, Clone)]
    pub struct DroneOrientations {
        entries: HashMap<u128, OrientationEntry>,
    }

    impl DroneOrientations {
        pub fn new() -> Self {
            Self {
                entries: HashMap::new(),
            }
        }

        pub fn set(&mut self, id: u128, mut q: Quat, now_nanos: u64) {
            if q.length_squared() < 1e-6 || !q.x.is_finite() || !q.y.is_finite() || !q.z.is_finite() || !q.w.is_finite() {
                return;
            }
            q = q.normalize();

            if let Some(entry) = self.entries.get_mut(&id) {
                entry.target = q;
            } else {
                self.entries.insert(id, OrientationEntry {
                    target: q,
                    display: q,
                    last_nanos: now_nanos,
                });
            }
        }

        pub fn has(&self, id: u128) -> bool {
            self.entries.contains_key(&id)
        }

        pub fn get(&mut self, id: u128, now_nanos: u64) -> Option<Quat> {
            let entry = self.entries.get_mut(&id)?;
            let dt = (now_nanos.saturating_sub(entry.last_nanos) as f32) * 1e-9;
            entry.last_nanos = now_nanos;

            let alpha = 1.0 - (-dt / SMOOTHING_SECONDS).exp();
            entry.display = entry.display.slerp(entry.target, alpha.clamp(0.0, 1.0));
            Some(entry.display)
        }

        pub fn remove(&mut self, id: u128) {
            self.entries.remove(&id);
        }

        pub fn clear(&mut self) {
            self.entries.clear();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_slerp_interpolation() {
            let mut orientations = DroneOrientations::new();
            let q1 = Quat::IDENTITY;
            let q2 = Quat::from_axis_angle(super::super::math::Vec3f::UP, std::f32::consts::PI / 2.0);

            orientations.set(123, q1, 1000);
            orientations.set(123, q2, 1000);

            let interpolated = orientations.get(123, 1000 + 40_000_000).unwrap();
            assert!(interpolated.y > 0.0);
        }
    }
}
pub mod physics {
    // Advanced physics engine for quadcopter aerodynamics, motor thermodynamics, and dynamics.
    // Direct port of AdvancedPhysicsCore.java.

    use super::build::{BuildParam, DroneBuildConfig};
    use super::math::{Quat, Vec3f};
    use super::physics_settings::{PhysicsConfig, PhysicsParam};
    use std::f32::consts::PI;

    pub const GRAVITY: f32 = 9.8;
    pub const INCHES_PER_METER: f32 = 39.3701;
    pub const REF_TEMP: f32 = 293.15;
    pub const VAC: f32 = 1.256637e-6;
    pub const MAX_TIME_STEP: f32 = 0.0078125;
    pub const MIN_TIME_STEP: f32 = 0.001;
    pub const PROP_SECTIONS: usize = 5;

    #[derive(Debug, Clone)]
    pub struct AdvancedPhysicsCore {
        applied_build_version: u32,
        applied_physics_version: u32,

        air_density: f32,
        drone_density: f32,
        air_drag_coefficient: f32,
        resistance_per_1000_ft: f32,
        temp_coeff: f32,
        specific_heat: f32,
        heat_transfer: f32,

        drone_mass: f32,
        battery_cells: i32,
        blades: i32,
        prop_radius: f32,
        prop_pitch: f32,
        prop_width: f32,
        motor_width: f32,
        motor_height: f32,
        motor_mass: f32,
        bell_mass: f32,
        blade_mass: f32,
        kv: f32,

        ref_resistance: f32,
        heat_cap: f32,
        motor_surface_area: f32,
        inertia: f32,
        drag_factor: f32,
        max_time_step: f32,
        hover_throttle: f32,

        pub internal_energy: [f32; 4],
        pub temp: [f32; 4],
        pub motor_vel: [f32; 4],
        pub motor_pos: [f32; 4],

        pub velocity: Vec3f,
        pub drone_up: Vec3f,
        pub drone_left: Vec3f,
    }

    impl Default for AdvancedPhysicsCore {
        fn default() -> Self {
            let mut core = Self {
                applied_build_version: 0,
                applied_physics_version: 0,
                air_density: 1.225,
                drone_density: 1500.0,
                air_drag_coefficient: 1.05,
                resistance_per_1000_ft: 25.7,
                temp_coeff: 0.00393,
                specific_heat: 400.0,
                heat_transfer: 50.0,
                drone_mass: 0.65,
                battery_cells: 2,
                blades: 3,
                prop_radius: 0.0635,
                prop_pitch: 0.1,
                prop_width: 0.015,
                motor_width: 0.023,
                motor_height: 0.006,
                motor_mass: 0.033,
                bell_mass: 0.012,
                blade_mass: 0.0015,
                kv: 2000.0 * 0.10471976,
                ref_resistance: 0.03,
                heat_cap: 13.2,
                motor_surface_area: 0.0016,
                inertia: 2e-5,
                drag_factor: 0.005,
                max_time_step: 0.005,
                hover_throttle: 0.5,
                internal_energy: [0.0; 4],
                temp: [REF_TEMP; 4],
                motor_vel: [0.0; 4],
                motor_pos: [0.0; 4],
                velocity: Vec3f::ZERO,
                drone_up: Vec3f::UP,
                drone_left: Vec3f::LEFT,
            };
            let b = DroneBuildConfig::default();
            let p = PhysicsConfig::default();
            core.apply_build_if_changed(&b, &p);
            core.reset_state(0.0);
            core
        }
    }

    impl AdvancedPhysicsCore {
        pub fn apply_build_if_changed(&mut self, build: &DroneBuildConfig, physics: &PhysicsConfig) {
            if build.version == self.applied_build_version && physics.version == self.applied_physics_version {
                return;
            }
            self.applied_build_version = build.version;
            self.applied_physics_version = physics.version;

            self.air_density = physics.get(PhysicsParam::AirDensity);
            self.drone_density = physics.get(PhysicsParam::DroneDensity);
            self.air_drag_coefficient = physics.get(PhysicsParam::DragCoefficient);
            self.resistance_per_1000_ft = physics.get(PhysicsParam::ResistancePer1000Ft);
            self.temp_coeff = physics.get(PhysicsParam::TempCoeff);
            self.specific_heat = physics.get(PhysicsParam::SpecificHeat);
            self.heat_transfer = physics.get(PhysicsParam::HeatTransfer);

            self.drone_mass = build.get(BuildParam::DroneMass);
            self.battery_cells = build.get(BuildParam::BatteryCells).round() as i32;
            self.blades = build.get(BuildParam::Blades).round() as i32;
            self.prop_radius = build.get(BuildParam::PropRadius);
            self.prop_pitch = build.get(BuildParam::PropPitch);
            self.prop_width = build.get(BuildParam::PropWidth);
            self.motor_width = build.get(BuildParam::MotorWidth);
            self.motor_height = build.get(BuildParam::MotorHeight);
            self.motor_mass = build.motor_mass();
            self.bell_mass = build.bell_mass();
            self.blade_mass = build.blade_mass();
            self.kv = build.get(BuildParam::MotorKv) * 0.10471976;

            let r = self.motor_width * 0.5;
            let poles = Self::get_poles(self.motor_width);
            let slots = Self::get_slots(self.motor_width);
            self.heat_cap = self.specific_heat * self.motor_mass;
            self.motor_surface_area = 2.0 * PI * r * self.motor_height + 2.0 * PI * r * r;

            let mut resistance = self.compute_ref_resistance(r, self.motor_height, self.kv, poles, slots);
            if resistance.is_nan() || resistance.is_infinite() || resistance < 0.001 {
                resistance = 0.03;
            }
            self.ref_resistance = resistance;
            self.inertia = self.compute_motor_inertia();

            let radius = (3.0 * self.drone_mass / (self.drone_density * 4.0 * PI)).cbrt();
            let area = PI * radius * radius;
            self.drag_factor = self.air_density * area * self.air_drag_coefficient / 2.0;

            let time_constant = self.ref_resistance * self.kv * self.kv * self.inertia;
            self.max_time_step = time_constant.clamp(MIN_TIME_STEP, MAX_TIME_STEP);
            self.hover_throttle = self.find_hover_throttle();
        }

        pub fn reset(&mut self, throttle: f32, build: &DroneBuildConfig, physics: &PhysicsConfig) {
            self.apply_build_if_changed(build, physics);
            self.reset_state(throttle);
        }

        pub fn get_hover_throttle(&mut self, build: &DroneBuildConfig, physics: &PhysicsConfig) -> f32 {
            self.apply_build_if_changed(build, physics);
            self.hover_throttle
        }

        pub fn reset_state(&mut self, throttle: f32) {
            for i in 0..4 {
                self.internal_energy[i] = 0.0;
                self.temp[i] = REF_TEMP;
                self.motor_vel[i] = 0.0;
                self.motor_pos[i] = 0.0;
            }
            self.velocity = Vec3f::ZERO;
            self.drone_up = Vec3f::UP;
            self.drone_left = Vec3f::LEFT;

            let vbat = self.get_battery_voltage(throttle.clamp(-1.0, 1.0));
            for motor in 0..4 {
                let use_vbat = (if motor % 2 == 0 { -1.0 } else { 1.0 }) * vbat;
                self.motor_vel[motor] = self.steady_motor_vel(motor, use_vbat, self.temp[motor]);
            }
        }

        fn steady_motor_vel(&self, motor: usize, vbat: f32, temp: f32) -> f32 {
            if vbat == 0.0 {
                return 0.0;
            }
            let direction = vbat.signum();
            let mut lo = 0.0;
            let mut hi = direction * self.kv * self.battery_cells as f32 * 4.2;
            for _ in 0..40 {
                let mid = (lo + hi) * 0.5;
                let accel = self.get_rot_accel(motor, mid, vbat, temp);
                if accel * direction > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            (lo + hi) * 0.5
        }

        pub fn step(&mut self, dt: f32, throttle: f32, orientation: Quat, vel: &mut Vec3f, build: &DroneBuildConfig, physics: &PhysicsConfig) {
            self.apply_build_if_changed(build, physics);

            self.drone_up = orientation.transform_vec3(Vec3f::UP);
            let drone_look = orientation.transform_vec3(Vec3f::new(0.0, 0.0, -1.0));
            self.drone_left = self.drone_up.cross(drone_look);
            self.velocity = *vel;

            let vbat = self.get_battery_voltage(throttle.clamp(-1.0, 1.0));
            let gravity_force = Vec3f::new(0.0, -GRAVITY * self.drone_mass, 0.0);

            let max_iterations = (dt / self.max_time_step) as usize + 10;
            let mut time_processed = 0.0;

            for _ in 0..max_iterations {
                if dt - time_processed <= 1e-5 {
                    break;
                }
                let time_step = (dt - time_processed).min(self.max_time_step);
                let speed = self.velocity.length();
                let mut net_force = self.velocity.normalized_or_zero().mul(-speed * speed * self.drag_factor).add(gravity_force);

                for motor in 0..4 {
                    let (thrust_force, _) = self.get_total_forces_on_prop(motor, self.motor_vel[motor]);
                    net_force = net_force.add(thrust_force);

                    let use_vbat = (if motor % 2 == 0 { -1.0 } else { 1.0 }) * vbat;
                    let next_motor_vel = self.advance_motor_vel(motor, self.motor_vel[motor], time_step, use_vbat, self.temp[motor]);
                    self.internal_energy[motor] = self.get_next_internal_energy(motor, self.internal_energy[motor], self.temp[motor], use_vbat, self.motor_vel[motor], time_step);
                    self.temp[motor] = self.get_next_temp(self.internal_energy[motor]);
                    self.motor_pos[motor] = Self::wrap_angle(self.motor_pos[motor] + self.motor_vel[motor] * time_step);
                    self.motor_vel[motor] = next_motor_vel;
                }

                self.velocity = self.velocity.fma(time_step / self.drone_mass, net_force);
                time_processed += time_step;
            }

            *vel = self.velocity;
        }

        pub fn is_overheat(&self) -> bool {
            let mut average_res = 0.0;
            for i in 0..4 {
                average_res += self.get_hot_resistance(self.temp[i]);
            }
            average_res /= 4.0;
            self.ref_resistance / average_res < 0.5
        }

        fn find_hover_throttle(&mut self) -> f32 {
            let weight = self.drone_mass * GRAVITY;
            let mut lo = 0.0;
            let mut hi = 1.0;
            for _ in 0..16 {
                let mid = (lo + hi) * 0.5;
                self.reset_state(mid);
                if self.total_thrust() < weight {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            self.reset_state(1.0);
            if self.total_thrust() < weight {
                return 1.0;
            }
            (lo + hi) * 0.5
        }

        fn total_thrust(&self) -> f32 {
            let mut thrust = 0.0;
            for motor in 0..4 {
                let (force, _) = self.get_total_forces_on_prop(motor, self.motor_vel[motor]);
                thrust += force.dot(self.drone_up);
            }
            thrust
        }

        fn get_motor_to_prop(rotation_angle: f32, world_drone_up: Vec3f, world_drone_right: Vec3f) -> Vec3f {
            Quat::from_axis_angle(world_drone_up, rotation_angle).transform_vec3(world_drone_right)
        }

        fn get_world_prop_velocity(world_drone_up: Vec3f, world_drone_velocity: Vec3f, motor_to_prop: Vec3f, angular_speed: f32, section_radius: f32) -> Vec3f {
            let prop_velocity = world_drone_up.cross(motor_to_prop).normalized_or_zero();
            prop_velocity.mul(angular_speed * section_radius).add(world_drone_velocity)
        }

        fn get_free_vel_rel_prop_planar(world_prop_velocity: Vec3f, world_velocity_free: Vec3f, motor_to_prop: Vec3f) -> Vec3f {
            let free_vel_rel_prop = world_velocity_free.sub(world_prop_velocity);
            let normal_proj = free_vel_rel_prop.project(motor_to_prop);
            free_vel_rel_prop.sub(normal_proj)
        }

        fn get_aoa(motor_number: usize, prop_pitch: f32, section_radius: f32, free_vel_rel_prop_planar: Vec3f, world_drone_up: Vec3f, motor_to_prop: Vec3f) -> f32 {
            let pitch_angle = prop_pitch.atan2(2.0 * PI * section_radius);
            let wind_direction = free_vel_rel_prop_planar.mul(-1.0).normalized_or_zero();
            let mut forward_spin_direction = world_drone_up.cross(motor_to_prop);
            if motor_number % 2 == 0 {
                forward_spin_direction = forward_spin_direction.mul(-1.0);
            }
            let mut sign = world_drone_up.dot(wind_direction).signum();
            if sign == 0.0 {
                sign = 1.0;
            }
            let wind_angle = sign * forward_spin_direction.angle_between(wind_direction);
            let mut angle_of_attack = pitch_angle - wind_angle;
            if motor_number % 2 == 0 {
                angle_of_attack *= -1.0;
            }
            angle_of_attack
        }

        fn get_prop_drag_coefficient(aoa: f32) -> f32 {
            let min: f32 = 0.4;
            let max: f32 = 1.8;
            let min_wrapped = min.powf(1.15);
            let max_wrapped = max.powf(1.15);
            let general_shape = -(aoa * 2.0).cos() + 1.0;
            let scaled_and_offset = general_shape / 2.0 * (max_wrapped - min_wrapped) + min_wrapped;
            scaled_and_offset.powf(0.8695652)
        }

        fn get_prop_lift_coefficient(mut aoa: f32) -> f32 {
            aoa = if aoa < 0.0 {
                PI + (aoa % PI)
            } else {
                aoa % PI
            };
            let alp = 7.5 * (PI / 180.0);
            let sqrt2_over_pi = (2.0 / PI).sqrt();
            let mut naca = sqrt2_over_pi * aoa * aoa * (-aoa * aoa / (2.0 * alp * alp)).exp() / (alp * alp * alp);
            let antiaoa = PI - aoa;
            let mut antinaca = -sqrt2_over_pi * antiaoa * antiaoa * (-antiaoa * antiaoa / (2.0 * alp * alp)).exp() / (alp * alp * alp);
            let plate = (aoa * 2.0).sin();
            naca /= 7.0;
            antinaca /= 7.0;
            plate + naca + antinaca
        }

        fn get_prop_drag_force(&self, free_vel_rel_prop_planar: Vec3f, drag_coefficient: f32, aoa: f32, section_length: f32, section_width: f32) -> Vec3f {
            let flow_speed = free_vel_rel_prop_planar.length();
            let thickness = 0.002;
            let reference_area = (section_length * section_width * aoa.sin()).abs() + (section_length * thickness * aoa.cos()).abs();
            let force_mag = drag_coefficient * self.air_density * flow_speed * flow_speed * reference_area / 2.0;
            free_vel_rel_prop_planar.normalized_or_zero().mul(force_mag)
        }

        fn get_prop_lift_force(&self, free_vel_rel_prop_planar: Vec3f, motor_to_prop: Vec3f, lift_coefficient: f32, section_length: f32, section_width: f32) -> Vec3f {
            let flow_speed = free_vel_rel_prop_planar.length();
            let reference_area = section_length * section_width;
            let force_mag = lift_coefficient * self.air_density * flow_speed * flow_speed * reference_area / 2.0;
            let wind_direction = free_vel_rel_prop_planar.mul(-1.0).normalized_or_zero();
            motor_to_prop.normalized_or_zero().cross(wind_direction).mul(force_mag)
        }

        fn get_total_forces_on_prop(&self, motor_number: usize, rotvel: f32) -> (Vec3f, Vec3f) {
            let mut net_force = Vec3f::ZERO;
            let mut net_drag_torque = Vec3f::ZERO;
            let world_velocity_free = Vec3f::ZERO;
            let drone_right = self.drone_left.mul(-1.0);
            let init_rotation_angle = self.motor_pos[motor_number];
            let section_length = self.prop_radius / PROP_SECTIONS as f32;

            for i in 0..self.blades {
                let rotation_angle = init_rotation_angle + 2.0 * PI * (i as f32) / (self.blades as f32);
                let motor_to_prop = Self::get_motor_to_prop(rotation_angle, self.drone_up, drone_right);

                for j in 0..PROP_SECTIONS {
                    let section_radius = ((j as f32) + 0.5) / (PROP_SECTIONS as f32) * self.prop_radius;
                    if section_radius < self.motor_width * 0.5 {
                        continue;
                    }
                    let mut section_width = self.prop_width;
                    if section_radius < self.motor_width {
                        section_width *= section_radius / self.motor_width;
                    }

                    let world_prop_velocity = Self::get_world_prop_velocity(self.drone_up, self.velocity, motor_to_prop, rotvel, section_radius);
                    let free_vel_rel_prop_planar = Self::get_free_vel_rel_prop_planar(world_prop_velocity, world_velocity_free, motor_to_prop);
                    let aoa = Self::get_aoa(motor_number, self.prop_pitch, section_radius, free_vel_rel_prop_planar, self.drone_up, motor_to_prop);
                    let drag_coeff = Self::get_prop_drag_coefficient(aoa);
                    let lift_coeff = Self::get_prop_lift_coefficient(aoa);

                    let drag_force = self.get_prop_drag_force(free_vel_rel_prop_planar, drag_coeff, aoa, section_length, section_width);
                    let lift_force = self.get_prop_lift_force(free_vel_rel_prop_planar, motor_to_prop, lift_coeff, section_length, section_width);

                    net_drag_torque = net_drag_torque.add(motor_to_prop.cross(drag_force).mul(section_radius));
                    net_force = net_force.add(drag_force).add(lift_force);
                }
            }

            (net_force, net_drag_torque)
        }

        fn dflux_dt(x: f32, slot_w: f32, r: f32, h: f32, m: f32, rotvel: f32) -> f32 {
            let gap = 0.00159;
            let xlow = -slot_w / 2.0;
            let xhigh = slot_w / 2.0;
            let gmin = (gap * gap + (x + xlow) * (x + xlow)) as f64;
            let gmax = (gap * gap + (x + xhigh) * (x + xhigh)) as f64;
            let c1 = (VAC * h * r * rotvel * m / (4.0 * PI)) as f64;
            let c2 = (3.0 * gap * gap) as f64;
            (c1 * (c2 * (gmax.powf(-2.5) - gmin.powf(-2.5)) - (gmax.powf(-1.5) - gmin.powf(-1.5)))) as f32
        }

        fn get_poles(motor_width: f32) -> f32 {
            if motor_width >= 0.018 { 14.0 } else { 12.0 }
        }

        fn get_slots(motor_width: f32) -> f32 {
            if motor_width >= 0.018 { 12.0 } else { 9.0 }
        }

        fn get_slot_w(r: f32, slots: f32) -> f32 {
            2.0 * PI * r / slots - 0.0005
        }

        fn get_mag_w(r: f32, poles: f32) -> f32 {
            2.0 * PI * r / poles - 0.0005
        }

        fn get_pole_magnetic_moment(r: f32, h: f32, poles: f32) -> f32 {
            let mag_w = Self::get_mag_w(r, poles);
            let mag_d = 0.001;
            1177746.6 * mag_w * h * mag_d
        }

        fn get_n(&self, r: f32, h: f32, kv: f32, poles: f32, slots: f32) -> f32 {
            let slot_w = Self::get_slot_w(r, slots);
            let mag_w = Self::get_mag_w(r, poles);
            let m = Self::get_pole_magnetic_moment(r, h, poles);
            let x1 = -mag_w + mag_w / 2.0;
            let x2 = mag_w / 2.0;
            let x3 = mag_w + mag_w / 2.0;
            let d1 = Self::dflux_dt(x1, slot_w, r, h, m, 1.0);
            let d2 = Self::dflux_dt(x2, slot_w, r, h, -m, 1.0);
            let d3 = Self::dflux_dt(x3, slot_w, r, h, m, 1.0);
            let d = d1 + d2 + d3;
            (1.0 / (d * kv)).round()
        }

        fn compute_ref_resistance(&self, r: f32, h: f32, kv: f32, poles: f32, slots: f32) -> f32 {
            let slot_w = Self::get_slot_w(r, slots);
            let n = self.get_n(r, h, kv, poles, slots);
            let loop_peri = 2.0 * h + 2.0 * slot_w;
            let wire_length_inches = n * loop_peri * INCHES_PER_METER;
            wire_length_inches / 12.0 / 1000.0 * self.resistance_per_1000_ft
        }

        fn compute_motor_inertia(&self) -> f32 {
            let blade_length = self.prop_radius;
            let prop_inertia = (self.blades as f32) / 12.0 * self.blade_mass * (4.0 * blade_length * blade_length + self.prop_width * self.prop_width);
            let r = self.motor_width * 0.5;
            let bell_inertia = self.bell_mass * r * r;
            prop_inertia + bell_inertia
        }

        fn get_hot_resistance(&self, temp: f32) -> f32 {
            self.ref_resistance * (1.0 + self.temp_coeff * (temp - REF_TEMP))
        }

        fn get_current(&self, resistance: f32, vbat: f32, rotvel: f32) -> f32 {
            let back_emf = -rotvel / self.kv;
            (vbat + back_emf) / resistance
        }

        fn get_torque(&self, current: f32) -> f32 {
            current / self.kv
        }

        fn get_rot_accel(&self, motor: usize, rotvel: f32, vbat: f32, temp: f32) -> f32 {
            let resistance = self.get_hot_resistance(temp);
            let current = self.get_current(resistance, vbat, rotvel);
            let torque = self.get_torque(current);
            let (_, drag_torque_vec) = self.get_total_forces_on_prop(motor, rotvel);
            let drag_torque = drag_torque_vec.dot(self.drone_up);
            (torque + drag_torque) / self.inertia
        }

        fn advance_motor_vel(&self, motor: usize, rotvel: f32, time_step: f32, vbat: f32, temp: f32) -> f32 {
            let k1 = self.get_rot_accel(motor, rotvel, vbat, temp);
            let k2 = self.get_rot_accel(motor, rotvel + time_step / 2.0 * k1, vbat, temp);
            let k3 = self.get_rot_accel(motor, rotvel + time_step / 2.0 * k2, vbat, temp);
            let k4 = self.get_rot_accel(motor, rotvel + time_step * k3, vbat, temp);

            let mut next_vel = rotvel + time_step * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
            if next_vel.is_infinite() || next_vel.is_nan() {
                next_vel = 0.0;
            }
            let max_speed = self.kv * self.battery_cells as f32 * 4.2;
            if next_vel.abs() > max_speed {
                next_vel = max_speed * next_vel.signum();
            }
            next_vel
        }

        fn get_battery_voltage(&self, throt: f32) -> f32 {
            let sag = 4.0 + throt.abs() * -0.4000001;
            sag * (self.battery_cells as f32) * throt
        }

        fn get_internal_energy_lost(&self, temp: f32, time_step: f32) -> f32 {
            let time_constant = self.heat_cap / (self.heat_transfer * self.motor_surface_area);
            let cooled_temp = REF_TEMP + (temp - REF_TEMP) * (-time_step / time_constant).exp();
            self.heat_cap * (cooled_temp - temp)
        }

        fn get_internal_energy_gained(&self, vbat: f32, rotvel: f32, time_step: f32) -> f32 {
            let current = self.get_current(self.ref_resistance, vbat, rotvel);
            let heating_power = current * current * self.ref_resistance;
            heating_power * time_step
        }

        fn get_next_internal_energy(&mut self, motor: usize, internal_energy: f32, temp_in: f32, vbat: f32, rotvel: f32, time_step: f32) -> f32 {
            let mut next_energy = internal_energy + self.get_internal_energy_lost(temp_in, time_step) + self.get_internal_energy_gained(vbat, rotvel, time_step);
            if next_energy.is_infinite() || next_energy.is_nan() {
                next_energy = 0.0;
                self.temp[motor] = 0.0;
            }
            next_energy
        }

        fn get_next_temp(&self, next_internal_energy: f32) -> f32 {
            next_internal_energy / self.heat_cap + REF_TEMP
        }

        fn wrap_angle(angle: f32) -> f32 {
            let two_pi = 2.0 * PI;
            let wrapped = angle % two_pi;
            if wrapped < 0.0 { wrapped + two_pi } else { wrapped }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_physics_initialization_and_hover() {
            let mut core = AdvancedPhysicsCore::default();
            let b = DroneBuildConfig::default();
            let p = PhysicsConfig::default();
            let hover = core.get_hover_throttle(&b, &p);
            assert!(hover > 0.1 && hover < 0.9, "Hover throttle expected in sensible range, got {}", hover);
        }

        #[test]
        fn test_physics_step_produces_motion() {
            let mut core = AdvancedPhysicsCore::default();
            let b = DroneBuildConfig::default();
            let p = PhysicsConfig::default();
            let mut vel = Vec3f::ZERO;
            let orient = Quat::IDENTITY;
            // Step with full throttle upwards
            for _ in 0..10 {
                core.step(0.02, 1.0, orient, &mut vel, &b, &p);
            }
            assert!(vel.y > 0.0, "Drone should accelerate upward under full throttle, got vel.y = {}", vel.y);
        }
    }
}
pub mod physics_settings {
    // Physical environment and motor thermal coefficients.
    // Direct port of PhysicsSettings.java.

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PhysicsParam {
        AirDensity,
        DroneDensity,
        DragCoefficient,
        ResistancePer1000Ft,
        TempCoeff,
        SpecificHeat,
        HeatTransfer,
    }

    impl PhysicsParam {
        pub const ALL: [PhysicsParam; 7] = [
            PhysicsParam::AirDensity,
            PhysicsParam::DroneDensity,
            PhysicsParam::DragCoefficient,
            PhysicsParam::ResistancePer1000Ft,
            PhysicsParam::TempCoeff,
            PhysicsParam::SpecificHeat,
            PhysicsParam::HeatTransfer,
        ];

        pub fn key(self) -> &'static str {
            match self {
                PhysicsParam::AirDensity => "air_density",
                PhysicsParam::DroneDensity => "drone_density",
                PhysicsParam::DragCoefficient => "drag_coefficient",
                PhysicsParam::ResistancePer1000Ft => "resistance_per_1000_ft",
                PhysicsParam::TempCoeff => "temp_coeff",
                PhysicsParam::SpecificHeat => "specific_heat",
                PhysicsParam::HeatTransfer => "heat_transfer",
            }
        }

        pub fn default_value(self) -> f32 {
            match self {
                PhysicsParam::AirDensity => 1.225,
                PhysicsParam::DroneDensity => 1500.0,
                PhysicsParam::DragCoefficient => 1.05,
                PhysicsParam::ResistancePer1000Ft => 25.7,
                PhysicsParam::TempCoeff => 0.00393,
                PhysicsParam::SpecificHeat => 400.0,
                PhysicsParam::HeatTransfer => 50.0,
            }
        }

        pub fn min(self) -> f32 {
            match self {
                PhysicsParam::AirDensity => 0.1,
                PhysicsParam::DroneDensity => 200.0,
                PhysicsParam::DragCoefficient => 0.1,
                PhysicsParam::ResistancePer1000Ft => 1.0,
                PhysicsParam::TempCoeff => 0.0,
                PhysicsParam::SpecificHeat => 100.0,
                PhysicsParam::HeatTransfer => 1.0,
            }
        }

        pub fn max(self) -> f32 {
            match self {
                PhysicsParam::AirDensity => 3.0,
                PhysicsParam::DroneDensity => 5000.0,
                PhysicsParam::DragCoefficient => 3.0,
                PhysicsParam::ResistancePer1000Ft => 100.0,
                PhysicsParam::TempCoeff => 0.01,
                PhysicsParam::SpecificHeat => 1000.0,
                PhysicsParam::HeatTransfer => 500.0,
            }
        }

        pub fn step(self) -> f32 {
            match self {
                PhysicsParam::AirDensity => 0.005,
                PhysicsParam::DroneDensity => 50.0,
                PhysicsParam::DragCoefficient => 0.01,
                PhysicsParam::ResistancePer1000Ft => 0.1,
                PhysicsParam::TempCoeff => 0.00001,
                PhysicsParam::SpecificHeat => 5.0,
                PhysicsParam::HeatTransfer => 1.0,
            }
        }

        pub fn snap(self, val: f32) -> f32 {
            let min = self.min();
            let max = self.max();
            let step = self.step();
            let steps = ((val - min) / step).round();
            (min + steps * step).clamp(min, max)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct PhysicsConfig {
        pub values: [f32; 7],
        pub version: u32,
    }

    impl Default for PhysicsConfig {
        fn default() -> Self {
            let mut vals = [0.0; 7];
            for (i, p) in PhysicsParam::ALL.iter().enumerate() {
                vals[i] = p.default_value();
            }
            Self { values: vals, version: 1 }
        }
    }

    impl PhysicsConfig {
        pub fn get(&self, param: PhysicsParam) -> f32 {
            self.values[param as usize]
        }

        pub fn set(&mut self, param: PhysicsParam, value: f32) {
            self.values[param as usize] = param.snap(value);
            self.version = self.version.wrapping_add(1);
        }

        pub fn reset(&mut self) {
            *self = Self::default();
            self.version = self.version.wrapping_add(1);
        }

        pub fn parse_properties(&mut self, text: &str) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    for param in PhysicsParam::ALL {
                        if param.key() == k {
                            if let Ok(num) = v.parse::<f32>() {
                                self.set(param, num);
                            }
                        }
                    }
                }
            }
        }

        pub fn to_properties(&self) -> String {
            let mut out = String::from("# FPV Drone physics constants, in SI units.\n");
            for param in PhysicsParam::ALL {
                out.push_str(&format!("{}={}\n", param.key(), self.get(param)));
            }
            out
        }
    }
}
pub mod registry {
    // Registry tracking active drone players across the world.
    // Port of DroneRegistry.java.

    use std::collections::HashSet;

    #[derive(Debug, Default, Clone)]
    pub struct DroneRegistry {
        active_drones: HashSet<u128>,
    }

    impl DroneRegistry {
        pub fn new() -> Self {
            Self {
                active_drones: HashSet::new(),
            }
        }

        pub fn is_drone(&self, id: u128) -> bool {
            self.active_drones.contains(&id)
        }

        pub fn set(&mut self, id: u128, active: bool) {
            if active {
                self.active_drones.insert(id);
            } else {
                self.active_drones.remove(&id);
            }
        }

        pub fn snapshot(&self) -> Vec<u128> {
            self.active_drones.iter().copied().collect()
        }

        pub fn clear(&mut self) {
            self.active_drones.clear();
        }
    }
}
pub mod state {
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
}

pub use build::{BuildParam, DroneBuildConfig};
pub use collision::CollisionResult;
pub use controller::{ControlAxis, ControllerConfig, VirtualStick};
pub use hud::{GimbalHud, HudPainter};
pub use math::{Quat, Vec3f};
pub use orientations::DroneOrientations;
pub use physics::AdvancedPhysicsCore;
pub use physics_settings::{PhysicsConfig, PhysicsParam};
pub use registry::DroneRegistry;
pub use state::DroneState;
