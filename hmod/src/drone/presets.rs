// Real drones to fly, so a player doesn't have to tune anything. Specs are the manufacturers'
// and retailers' published figures (all-up weight with the usual battery); `efficiency` is
// calibrated so each flies with its class's real thrust-to-weight (see the tests).

use super::build::{BuildParam, DroneBuildConfig};

/// The look of a drone: which model draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// Open X frame with bare props (5" and 7").
    X,
    /// Props inside ducts (cinewhoops and tiny whoops).
    Ducted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub class: &'static str,
    pub description: &'static str,
    pub frame: Frame,
    /// Motor-to-motor diagonal, metres (the wheelbase).
    pub wheelbase: f32,
    pub mass: f32,
    pub cells: f32,
    pub kv: f32,
    pub blades: f32,
    pub prop_radius: f32,
    pub prop_pitch: f32,
    pub prop_width: f32,
    pub stator_width: f32,
    pub stator_height: f32,
    pub battery_mah: f32,
    pub camera_angle: f32,
    pub efficiency: f32,
    /// The thrust-to-weight its class really flies with, which `efficiency` is tuned to.
    pub target_twr: f32,
}

impl Preset {
    pub fn build(&self) -> DroneBuildConfig {
        let mut b = DroneBuildConfig::default();
        for (param, value) in [
            (BuildParam::DroneMass, self.mass),
            (BuildParam::BatteryCells, self.cells),
            (BuildParam::MotorKv, self.kv),
            (BuildParam::Blades, self.blades),
            (BuildParam::PropRadius, self.prop_radius),
            (BuildParam::PropPitch, self.prop_pitch),
            (BuildParam::PropWidth, self.prop_width),
            (BuildParam::MotorWidth, self.stator_width),
            (BuildParam::MotorHeight, self.stator_height),
            (BuildParam::CameraAngle, self.camera_angle),
            (BuildParam::Efficiency, self.efficiency),
            (BuildParam::BatteryMah, self.battery_mah),
        ] {
            b.set(param, value);
        }
        b
    }
}

pub const PRESETS: [Preset; 5] = [
    Preset {
        id: "nazgul5",
        name: "iFlight Nazgul5 V3",
        class: "5\" Freestyle, 6S",
        description: "The classic 5-inch freestyle quad: XING-E Pro 2207 1800KV motors on 6S, \
            5.1\" tri-blades, 245 mm wheelbase, about 660 g with a 6S 1400 mAh pack. Punchy, \
            stable and forgiving: the drone most pilots learn freestyle on.",
        frame: Frame::X,
        wheelbase: 0.245,
        mass: 0.66,
        cells: 6.0,
        kv: 1800.0,
        blades: 3.0,
        prop_radius: 0.0648,
        prop_pitch: 0.104,
        prop_width: 0.015,
        stator_width: 0.022,
        stator_height: 0.007,
        battery_mah: 1400.0,
        camera_angle: 30.0,
        efficiency: 0.425,
        target_twr: 9.0,
    },
    Preset {
        id: "xl5",
        name: "iFlight XL5 V5",
        class: "5\" Race, 4S",
        description: "A light 240 mm 5-inch racer: XING-E 2207 2750KV motors on 4S, about 400 g \
            dry and 580 g with a 4S 1500 mAh pack. Quick to rev and quick to turn, for gates \
            and fast laps.",
        frame: Frame::X,
        wheelbase: 0.240,
        mass: 0.58,
        cells: 4.0,
        kv: 2750.0,
        blades: 3.0,
        prop_radius: 0.0635,
        prop_pitch: 0.109,
        prop_width: 0.015,
        stator_width: 0.022,
        stator_height: 0.007,
        battery_mah: 1500.0,
        camera_angle: 35.0,
        efficiency: 0.405,
        target_twr: 8.0,
    },
    Preset {
        id: "cinelog35",
        name: "GEPRC CineLog35 V2",
        class: "3.5\" Cinewhoop, 6S",
        description: "A ducted 3.5-inch cinewhoop for smooth flying close to things: SPEEDX2 \
            2105.5 2650KV motors on 6S, 90 mm props in guards, 142 mm wheelbase, about 435 g \
            with a 6S 1050 mAh pack. Bumps off walls instead of breaking props.",
        frame: Frame::Ducted,
        wheelbase: 0.142,
        mass: 0.435,
        cells: 6.0,
        kv: 2650.0,
        blades: 5.0,
        prop_radius: 0.045,
        prop_pitch: 0.061,
        prop_width: 0.010,
        stator_width: 0.021,
        stator_height: 0.0055,
        battery_mah: 1050.0,
        camera_angle: 15.0,
        efficiency: 0.195,
        target_twr: 5.5,
    },
    Preset {
        id: "meteor75",
        name: "BetaFPV Meteor75 Pro",
        class: "75 mm Tiny Whoop, 1S",
        description: "A 45 g indoor whoop: 1102 22000KV motors on a single 550 mAh cell, 45 mm \
            tri-blades in ducts. Slow, floaty and almost impossible to break: fly it through \
            doorways and around the house.",
        frame: Frame::Ducted,
        wheelbase: 0.075,
        mass: 0.0453,
        cells: 1.0,
        kv: 22000.0,
        blades: 3.0,
        prop_radius: 0.0225,
        prop_pitch: 0.025,
        prop_width: 0.006,
        stator_width: 0.011,
        stator_height: 0.002,
        battery_mah: 550.0,
        camera_angle: 10.0,
        efficiency: 0.265,
        target_twr: 3.5,
    },
    Preset {
        id: "chimera7",
        name: "iFlight Chimera7 Pro V2",
        class: "7.5\" Long Range, 6S",
        description: "A 7.5-inch long-range cruiser: XING2 2809 1250KV motors on 6S, HQ 7.5\" \
            props, 327 mm wheelbase, about 1165 g with a 6S 3300 mAh pack. Heavy and smooth, \
            with long flight times for mountain dives and distance.",
        frame: Frame::X,
        wheelbase: 0.327,
        mass: 1.165,
        cells: 6.0,
        kv: 1250.0,
        blades: 3.0,
        prop_radius: 0.0953,
        prop_pitch: 0.094,
        prop_width: 0.019,
        stator_width: 0.028,
        stator_height: 0.009,
        battery_mah: 3300.0,
        camera_angle: 25.0,
        efficiency: 0.265,
        target_twr: 7.0,
    },
];

/// The preset with `id`, or the first.
pub fn by_id(id: &str) -> &'static Preset {
    PRESETS.iter().find(|p| p.id == id).unwrap_or(&PRESETS[0])
}

/// How `build` flies in standard air: its thrust-to-weight at full throttle, and the throttle
/// that holds it in a hover (0..1).
pub fn performance(build: &DroneBuildConfig) -> (f32, f32) {
    use super::physics::{AdvancedPhysicsCore, GRAVITY};
    use super::physics_settings::PhysicsConfig;
    let p = PhysicsConfig::default();
    let mut core = AdvancedPhysicsCore::default();
    let hover = core.get_hover_throttle(build, &p);
    core.reset(1.0, build, &p);
    (core.static_thrust() / (build.get(BuildParam::DroneMass) * GRAVITY), hover)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn twr_and_hover(build: &DroneBuildConfig) -> (f32, f32) {
        performance(build)
    }

    #[test]
    fn presets_fly_like_their_class() {
        for preset in &PRESETS {
            let (twr, hover) = twr_and_hover(&preset.build());
            eprintln!("{}: twr {twr:.2} (target {}), hover {hover:.3}", preset.id, preset.target_twr);
            assert!(
                (twr / preset.target_twr - 1.0).abs() < 0.15,
                "{} thrust-to-weight {twr:.2}, its class flies at {}",
                preset.id,
                preset.target_twr
            );
            assert!(hover > 0.1 && hover < 0.6, "{} hovers at {hover}", preset.id);
        }
    }

    /// Prints the efficiency that gives each preset its target thrust-to-weight.
    #[test]
    #[ignore = "calibration helper: cargo test calibrate -- --ignored --nocapture"]
    fn calibrate() {
        for preset in &PRESETS {
            let (mut lo, mut hi) = (0.05_f32, 1.5_f32);
            for _ in 0..30 {
                let mid = (lo + hi) / 2.0;
                let mut b = preset.build();
                b.set(BuildParam::Efficiency, mid);
                if twr_and_hover(&b).0 < preset.target_twr { lo = mid } else { hi = mid }
            }
            eprintln!("{}: efficiency {:.3}", preset.id, (lo + hi) / 2.0);
        }
    }
}
