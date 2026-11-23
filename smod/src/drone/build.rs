// Drone hardware specification and physical dimensions.
// Based on DroneBuild.java from Minecraft FPV (GPL-3.0, github.com/minecraft-fpv/mfpv-client).

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
    /// How much of the blade-element model's lift the props really make: frame blockage,
    /// ducts and prop quality. Calibrates a build to its real thrust-to-weight.
    Efficiency,
    /// The battery pack's capacity, mAh.
    BatteryMah,
    /// Motor-to-motor diagonal, metres: how far out the motors sit, which sets how hard the
    /// drone is to turn (its moment of inertia) and the leverage its motors have.
    Wheelbase,
}

impl BuildParam {
    pub const ALL: [BuildParam; 13] = [
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
        BuildParam::Efficiency,
        BuildParam::BatteryMah,
        BuildParam::Wheelbase,
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
            BuildParam::Efficiency => "efficiency",
            BuildParam::BatteryMah => "battery_mah",
            BuildParam::Wheelbase => "wheelbase",
        }
    }

    pub fn default_value(self) -> f32 {
        match self {
            BuildParam::DroneMass => 0.65,
            BuildParam::BatteryCells => 4.0,
            BuildParam::MotorKv => 2400.0,
            BuildParam::Blades => 3.0,
            BuildParam::PropRadius => 0.0635,
            BuildParam::PropPitch => 0.1,
            BuildParam::PropWidth => 0.015,
            BuildParam::MotorWidth => 0.023,
            BuildParam::MotorHeight => 0.006,
            BuildParam::CameraAngle => 10.0,
            BuildParam::Efficiency => 0.42,
            BuildParam::BatteryMah => 1400.0,
            BuildParam::Wheelbase => 0.22,
        }
    }

    pub fn min(self) -> f32 {
        match self {
            BuildParam::DroneMass => 0.02,
            BuildParam::BatteryCells => 1.0,
            BuildParam::MotorKv => 500.0,
            BuildParam::Blades => 2.0,
            BuildParam::PropRadius => 0.015,
            BuildParam::PropPitch => 0.03,
            BuildParam::PropWidth => 0.003,
            BuildParam::MotorWidth => 0.01,
            BuildParam::MotorHeight => 0.0015,
            BuildParam::CameraAngle => 0.0,
            BuildParam::Efficiency => 0.1,
            BuildParam::BatteryMah => 100.0,
            BuildParam::Wheelbase => 0.04,
        }
    }

    pub fn max(self) -> f32 {
        match self {
            BuildParam::DroneMass => 2.0,
            BuildParam::BatteryCells => 8.0,
            BuildParam::MotorKv => 30000.0,
            BuildParam::Blades => 6.0,
            BuildParam::PropRadius => 0.15,
            BuildParam::PropPitch => 0.25,
            BuildParam::PropWidth => 0.04,
            BuildParam::MotorWidth => 0.06,
            BuildParam::MotorHeight => 0.03,
            BuildParam::CameraAngle => 70.0,
            BuildParam::Efficiency => 1.5,
            BuildParam::BatteryMah => 10000.0,
            BuildParam::Wheelbase => 0.8,
        }
    }

    pub fn step(self) -> f32 {
        match self {
            BuildParam::DroneMass => 0.001,
            BuildParam::BatteryCells => 1.0,
            BuildParam::MotorKv => 50.0,
            BuildParam::Blades => 1.0,
            BuildParam::PropRadius => 0.0005,
            BuildParam::PropPitch => 0.001,
            BuildParam::PropWidth => 0.0005,
            BuildParam::MotorWidth => 0.0005,
            BuildParam::MotorHeight => 0.0001,
            BuildParam::CameraAngle => 1.0,
            BuildParam::Efficiency => 0.01,
            BuildParam::BatteryMah => 50.0,
            BuildParam::Wheelbase => 0.001,
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
    pub values: [f32; 13],
    pub version: u32,
}

impl Default for DroneBuildConfig {
    fn default() -> Self {
        let mut vals = [0.0; 13];
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
        let mut out = String::from("# FPV Horizon build config. SI units: kg, m, rpm/V.\n");
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
        assert_eq!(cfg.get(BuildParam::BatteryCells), 4.0);
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
