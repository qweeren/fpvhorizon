// Betaflight rate model, stick shaping, and flight control inputs.
// Based on DroneController.java and VirtualJoystick.java from Minecraft FPV (GPL-3.0, github.com/minecraft-fpv/mfpv-client).

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

pub const DEFAULT_RC_RATE: f32 = 1.0;
pub const DEFAULT_SUPER_RATE: f32 = 0.7;
pub const DEFAULT_EXPO: f32 = 0.0;
pub const DEFAULT_DEADZONE: f32 = 0.05;

/// Angle and horizon modes: the furthest the sticks tilt the drone, degrees (Betaflight's
/// `angle_limit`; Liftoff's level angle limit).
pub const ANGLE_LIMIT_MIN: f32 = 10.0;
pub const ANGLE_LIMIT_MAX: f32 = 85.0;
pub const DEFAULT_ANGLE_LIMIT: f32 = 55.0;
/// How hard self-levelling pulls toward the stick's angle (Betaflight's level P, `angle_strength`).
pub const LEVEL_STRENGTH_MIN: f32 = 10.0;
pub const LEVEL_STRENGTH_MAX: f32 = 200.0;
pub const DEFAULT_LEVEL_STRENGTH: f32 = 50.0;

/// The flight controller's mode: what the roll and pitch sticks command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlightMode {
    /// Rate mode: the sticks command rotation rates and the drone holds whatever attitude it's left at.
    #[default]
    Acro,
    /// Angle (level) mode: the sticks command a tilt, up to the angle limit, and centred sticks level it.
    Angle,
    /// Self-levels near centre stick, flies like acro at full stick (flips and rolls allowed).
    Horizon,
    /// Acro with reversible motors: the lower half of the throttle thrusts the other way.
    Acro3d,
}

impl FlightMode {
    /// In the order the mode switch (L1, M) steps through them.
    pub const ALL: [FlightMode; 4] = [FlightMode::Acro, FlightMode::Angle, FlightMode::Horizon, FlightMode::Acro3d];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Self {
        Self::ALL.get(i).copied().unwrap_or_default()
    }

    pub fn next(self) -> Self {
        Self::from_index((self.index() + 1) % Self::ALL.len())
    }

    pub fn key(self) -> &'static str {
        match self {
            FlightMode::Acro => "acro",
            FlightMode::Angle => "angle",
            FlightMode::Horizon => "horizon",
            FlightMode::Acro3d => "3d",
        }
    }

    /// As the setup screen names it.
    pub fn name(self) -> &'static str {
        match self {
            FlightMode::Acro => "Acro",
            FlightMode::Angle => "Angle (level)",
            FlightMode::Horizon => "Horizon",
            FlightMode::Acro3d => "3D",
        }
    }

    /// As a Betaflight OSD shows it.
    pub fn osd(self) -> &'static str {
        match self {
            FlightMode::Acro => "ACRO",
            FlightMode::Angle => "ANGL",
            FlightMode::Horizon => "HOR",
            FlightMode::Acro3d => "3D",
        }
    }

    pub fn is_3d(self) -> bool {
        self == FlightMode::Acro3d
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControllerConfig {
    pub deadzone: f32,
    pub rc_rate: [f32; 4],
    pub super_rate: [f32; 4],
    pub expo: [f32; 4],
    pub flight_mode: FlightMode,
    pub angle_limit: f32,
    pub level_strength: f32,
    /// A self-centring throttle stick (a gamepad): centre is hover throttle, not 50%.
    pub throttle_centre_hover: bool,
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
            flight_mode: FlightMode::Acro,
            angle_limit: DEFAULT_ANGLE_LIMIT,
            level_strength: DEFAULT_LEVEL_STRENGTH,
            throttle_centre_hover: true,
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
                } else if k == "flight_mode" {
                    if let Some(m) = FlightMode::ALL.into_iter().find(|m| m.key() == v) { self.flight_mode = m; }
                } else if k == "mode_3d" {
                    // Before flight modes: 3D was its own switch.
                    if v.eq_ignore_ascii_case("true") { self.flight_mode = FlightMode::Acro3d; }
                } else if k == "angle_limit" {
                    if let Ok(num) = v.parse::<f32>() { self.angle_limit = num.clamp(ANGLE_LIMIT_MIN, ANGLE_LIMIT_MAX); }
                } else if k == "level_strength" {
                    if let Ok(num) = v.parse::<f32>() { self.level_strength = num.clamp(LEVEL_STRENGTH_MIN, LEVEL_STRENGTH_MAX); }
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
        let mut out = String::from("# FPV Horizon controller config\n");
        out.push_str(&format!("deadzone={}\n", self.deadzone));
        out.push_str(&format!("flight_mode={}\n", self.flight_mode.key()));
        out.push_str(&format!("angle_limit={}\n", self.angle_limit));
        out.push_str(&format!("level_strength={}\n", self.level_strength));
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
