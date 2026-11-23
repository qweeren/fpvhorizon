//! The mod's settings, kept by the game: registered for its page in Options > Mods from the
//! setup screen's table (`setup::model`), and read here into the drone. The drone is a real
//! preset unless "Custom Build" is on, which hands the build to the sliders. Rates, sound and
//! the physics constants apply to any.

use crate::drone::{self, BuildParam, ControlAxis, DroneState, PhysicsParam, PRESETS};
use crate::setup::model::{self, Kind};
use mod_api::{ClientHost, KeyRegistration, Registrar, SettingSpec};

pub const MOD_ID: &str = "fpvhorizon";
pub use model::FISHEYE_DEFAULT;
/// The setup screen's key mapping, and the key it starts on.
pub const SETUP_KEY: &str = "setup";
const SETUP_KEY_DEFAULT: &str = "key.keyboard.j";
/// The rate axes on the page (throttle has no rate).
const AXES: [ControlAxis; 3] = model::RATE_AXES;

pub fn register(r: &mut dyn Registrar) {
    for s in model::settings() {
        let label = format!("fpvhorizon.setting.{}", s.key);
        let spec = match &s.kind {
            Kind::Range { min, max, step, default, .. } => SettingSpec::range(&s.key, label, *min, *max, *step, *default),
            Kind::Toggle { default } => SettingSpec::toggle(&s.key, label, *default),
            Kind::Choice { options, default } => {
                SettingSpec::choice(&s.key, label, options.iter().map(|(key, _)| key.clone()), *default)
            }
        };
        let _ = r.register_setting(spec);
    }
    r.register_key_mapping(KeyRegistration::new(SETUP_KEY, SETUP_KEY_DEFAULT).label("fpvhorizon.key.setup"));
}

/// The setup key's name as Controls shows it: `J`, or what the player bound it to.
pub fn setup_key_name(host: &dyn ClientHost) -> String {
    let key = host.mod_key(MOD_ID, SETUP_KEY).map_or_else(|| SETUP_KEY_DEFAULT.to_string(), |(key, _)| key);
    key_label(&key)
}

/// `key.keyboard.left.bracket` as a person writes it: `Left Bracket`; a letter in capitals.
fn key_label(key: &str) -> String {
    let words = |name: &str| {
        name.split('.')
            .map(|w| {
                let mut c = w.chars();
                c.next().map_or_else(String::new, |first| first.to_ascii_uppercase().to_string() + c.as_str())
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    if let Some(button) = key.strip_prefix("key.mouse.") {
        return format!("Mouse {}", words(button));
    }
    let name = key.strip_prefix("key.keyboard.").unwrap_or(key);
    if name.chars().count() == 1 { name.to_uppercase() } else { words(name) }
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

/// Keeps the flight mode switched in flight, so the page and the next session agree.
pub fn choose_flight_mode(host: &dyn ClientHost, mode: drone::FlightMode) {
    host.set_setting(MOD_ID, "flight_mode", &mode.index().to_string());
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
    let battery = host.setting_bool("battery_sim").unwrap_or(true);
    let heat = host.setting_bool("heat_sim").unwrap_or(true);
    state.physics.set_simulation(battery, heat);
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
    if let Some(i) = host.setting_choice("flight_mode") {
        cfg.flight_mode = drone::FlightMode::from_index(i as usize);
    }
    if let Some(v) = f(host, "angle_limit") {
        cfg.angle_limit = v;
    }
    if let Some(v) = f(host, "level_strength") {
        cfg.level_strength = v;
    }
    if let Some(on) = host.setting_bool("throttle_centre_hover") {
        cfg.throttle_centre_hover = on;
    }
}
