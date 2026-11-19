//! The mod's page in Options > Mods. The drone is a real preset unless "Custom Build" is on,
//! which hands the build to the sliders. Rates, sound and the physics constants apply to any.

use crate::drone::{self, BuildParam, ControlAxis, DroneState, PhysicsParam, PRESETS};
use mod_api::{ClientHost, Registrar, SettingSpec};

pub const MOD_ID: &str = "fpvdrone";
/// The fisheye lens's choice by default: Medium.
pub const FISHEYE_DEFAULT: usize = 2;
/// The rate axes on the page (throttle has no rate).
const AXES: [ControlAxis; 3] = [ControlAxis::Roll, ControlAxis::Pitch, ControlAxis::Yaw];

fn label(key: &str) -> String {
    format!("fpvdrone.setting.{key}")
}

fn range(key: &str, min: f64, max: f64, step: f64, default: f64) -> SettingSpec {
    SettingSpec::range(key, label(key), min, max, step, default)
}

pub fn register(r: &mut dyn Registrar) {
    use drone::controller as c;
    let mut specs = vec![
        SettingSpec::choice("drone", label("drone"), PRESETS.iter().map(|p| format!("fpvdrone.preset.{}", p.id)), 0),
        SettingSpec::toggle("custom_build", label("custom_build"), false),
        SettingSpec::toggle("hide_hud", label("hide_hud"), true),
        SettingSpec::toggle("show_sticks", label("show_sticks"), true),
        range("fpv_fov", 60.0, 150.0, 1.0, 110.0),
        SettingSpec::choice(
            "fisheye",
            label("fisheye"),
            ["off", "light", "medium", "strong"].map(|l| format!("fpvdrone.fisheye.{l}")),
            FISHEYE_DEFAULT,
        ),
        range("mouse_sensitivity", 0.1, 5.0, 0.05, 1.0),
        range("gravity", 4.0, 30.0, 0.1, 9.8),
        SettingSpec::toggle("throttle_centre_hover", label("throttle_centre_hover"), true),
        SettingSpec::toggle("mode_3d", label("mode_3d"), false),
        range("deadzone", 0.0, 0.3, 0.01, f64::from(c::DEFAULT_DEADZONE)),
        range("range", 32.0, 512.0, 8.0, 160.0),
        range("motor_volume", 0.0, 1.0, 0.05, 0.6),
        range("wind_volume", 0.0, 1.0, 0.05, 0.8),
    ];
    for axis in AXES {
        let a = axis.key();
        specs.push(range(&format!("rate_{a}_rc"), f64::from(c::RC_RATE_MIN), f64::from(c::RC_RATE_MAX), 0.01, f64::from(c::DEFAULT_RC_RATE)));
        specs.push(range(&format!("rate_{a}_super"), f64::from(c::SUPER_RATE_MIN), f64::from(c::SUPER_RATE_MAX), 0.01, f64::from(c::DEFAULT_SUPER_RATE)));
        specs.push(range(&format!("rate_{a}_expo"), f64::from(c::EXPO_MIN), f64::from(c::EXPO_MAX), 0.01, f64::from(c::DEFAULT_EXPO)));
    }
    for p in BuildParam::ALL {
        specs.push(range(&format!("build_{}", p.key()), f64::from(p.min()), f64::from(p.max()), f64::from(p.step()), f64::from(p.default_value())));
    }
    for p in PhysicsParam::ALL {
        specs.push(range(&format!("physics_{}", p.key()), f64::from(p.min()), f64::from(p.max()), f64::from(p.step()), f64::from(p.default_value())));
    }
    for spec in specs {
        let _ = r.register_setting(spec);
    }
}

pub fn f(host: &dyn ClientHost, key: &str) -> Option<f32> {
    host.setting_f64(key).map(|v| v as f32)
}

/// The preset chosen on the settings page.
pub fn chosen_preset(host: &dyn ClientHost) -> usize {
    host.setting_choice("drone").map_or(0, |i| i as usize).min(PRESETS.len() - 1)
}

/// Keeps the preset chosen in flight, so the page and the next session agree.
pub fn choose_preset(host: &dyn ClientHost, index: usize) {
    host.set_setting(MOD_ID, "drone", &index.to_string());
}

/// The build the drone should have: preset `preset`, or the sliders' with Custom Build on.
pub fn build(host: &dyn ClientHost, preset: usize) -> drone::DroneBuildConfig {
    let mut b = PRESETS[preset].build();
    if host.setting_bool("custom_build").unwrap_or(false) {
        for p in BuildParam::ALL {
            if let Some(v) = f(host, &format!("build_{}", p.key())) {
                b.set(p, v);
            }
        }
    }
    b
}

/// Reads everything but the build into the drone.
pub fn apply(host: &dyn ClientHost, state: &mut DroneState) {
    for p in PhysicsParam::ALL {
        if let Some(v) = f(host, &format!("physics_{}", p.key()))
            && p.snap(v) != state.physics_config.get(p)
        {
            state.physics_config.set(p, v);
        }
    }
    if let Some(g) = f(host, "gravity") {
        state.physics.set_gravity(g);
    }
    let cfg = &mut state.controller_config;
    for axis in AXES {
        let (a, i) = (axis.key(), axis as usize);
        if let Some(v) = f(host, &format!("rate_{a}_rc")) {
            cfg.rc_rate[i] = v;
        }
        if let Some(v) = f(host, &format!("rate_{a}_super")) {
            cfg.super_rate[i] = v;
        }
        if let Some(v) = f(host, &format!("rate_{a}_expo")) {
            cfg.expo[i] = v;
        }
    }
    if let Some(v) = f(host, "deadzone") {
        cfg.deadzone = v;
    }
    if let Some(on) = host.setting_bool("mode_3d") {
        cfg.mode_3d = on;
    }
    if let Some(on) = host.setting_bool("throttle_centre_hover") {
        cfg.throttle_centre_hover = on;
    }
}
