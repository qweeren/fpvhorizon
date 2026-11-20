//! What the setup screen shows, without the game: every setting the mod keeps (its key, range,
//! default, unit and words), the pages they sit on, and the maths behind the sliders, the rate
//! curves, the fisheye preview and the video link. The settings page in Options > Mods is
//! registered from the same table, so both always agree. Unit-tested with the drone core.

use crate::drone::controller as c;
use crate::drone::{BuildParam, ControlAxis, ControllerConfig, PhysicsParam, PRESETS};
use std::sync::LazyLock;

/// The fisheye lens's choice by default: Medium.
pub const FISHEYE_DEFAULT: usize = 2;
/// How much each fisheye choice magnifies the middle against the corners (the post chains'
/// `Strength`, `assets/fpvhorizon/post_effect/fisheye_*.json`), Off first.
pub const FISHEYE_STRENGTH: [f32; 4] = [0.0, 0.25, 0.5, 0.9];

/// The video link is at full strength out to this share of the range, then fades in a straight
/// line to nothing at the range.
pub const LINK_FADE_START: f64 = 0.3;
/// Link quality below which the LOW SIGNAL badge shows.
pub const LINK_LOW: f32 = 40.0;
/// Link quality below which the badge turns red and says to turn back.
pub const LINK_CRITICAL: f32 = 15.0;

/// The video range's top stop: no range limit, the link never fades.
pub const RANGE_UNLIMITED: f64 = 520.0;

/// Whether `range` (the setting's value) is the top stop, with no limit.
pub fn range_unlimited(range: f64) -> bool {
    range >= RANGE_UNLIMITED
}

/// Link quality, 0..100, at `distance` blocks from the pilot with the link lost at `range`
/// (always full with no range limit).
pub fn link_quality(distance: f64, range: f64) -> f32 {
    if range_unlimited(range) {
        return 100.0;
    }
    let x = distance / range.max(1.0);
    (100.0 * ((1.0 - x) / (1.0 - LINK_FADE_START)).clamp(0.0, 1.0)) as f32
}

/// How far from the pilot the link falls to `quality`, with the link lost at `range`.
pub fn distance_at_quality(quality: f32, range: f64) -> f64 {
    range * (1.0 - f64::from(quality) / 100.0 * (1.0 - LINK_FADE_START))
}

/// How a number is written on the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unit {
    /// A plain number with this many decimals.
    Num(u8),
    /// A number with this many decimals and a unit after it.
    Suffix(u8, &'static str),
    /// A share, 0..1, as a percentage.
    Percent,
    /// A factor: `1.00x`.
    Times,
    /// Kilograms, written in grams.
    Grams,
    /// Battery cells in series: `6S`.
    Cells,
    /// A radius in metres, written as the diameter in inches (as props are sold).
    DiameterInches,
    /// Metres, written in inches.
    Inches,
    /// Metres, written in millimetres with this many decimals.
    Millimetres(u8),
    /// The video range in metres, its top stop written as no limit.
    Range,
}

impl Unit {
    pub fn format(self, v: f64) -> String {
        match self {
            Unit::Num(d) => format!("{v:.*}", usize::from(d)),
            Unit::Suffix(d, s) => format!("{v:.*}{s}", usize::from(d)),
            Unit::Percent => format!("{:.0}%", v * 100.0),
            Unit::Times => format!("{v:.2}x"),
            Unit::Grams => format!("{:.0} g", v * 1000.0),
            Unit::Cells => format!("{v:.0}S"),
            Unit::DiameterInches => format!("{:.1} in", v * 2.0 / 0.0254),
            Unit::Inches => format!("{:.1} in", v / 0.0254),
            Unit::Millimetres(d) => format!("{:.*} mm", usize::from(d), v * 1000.0),
            Unit::Range if range_unlimited(v) => "Unlimited".to_string(),
            Unit::Range => format!("{v:.0} m"),
        }
    }
}

/// A setting's value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Range(f64),
    Toggle(bool),
    Choice(usize),
}

/// What a setting is, with its default.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Range { min: f64, max: f64, step: f64, default: f64, unit: Unit },
    Toggle { default: bool },
    /// One of `options`: each its translation key (for Options > Mods) and its words here.
    Choice { options: Vec<(String, String)>, default: usize },
}

/// One of the mod's settings.
#[derive(Clone, Debug, PartialEq)]
pub struct Setting {
    pub key: String,
    /// Its name on the setup screen.
    pub label: String,
    /// One line on what it does, shown while the pointer is on it.
    pub help: &'static str,
    pub kind: Kind,
}

/// The number of decimals in `step` (at most 6), so a snapped value is written without float
/// noise (`0.25`, not `0.25000000000000006`).
fn decimals(step: f64) -> i32 {
    (0..6).find(|&d| {
        let scaled = step * 10f64.powi(d);
        (scaled - scaled.round()).abs() < 1e-6
    })
    .unwrap_or(6)
}

impl Setting {
    pub fn default_value(&self) -> Value {
        match self.kind {
            Kind::Range { default, .. } => Value::Range(default),
            Kind::Toggle { default } => Value::Toggle(default),
            Kind::Choice { default, .. } => Value::Choice(default),
        }
    }

    /// `value` as the game keeps it (`SettingSpec::read` takes it back), or None when it is
    /// not one of this setting's.
    pub fn encode(&self, value: Value) -> Option<String> {
        match (&self.kind, value) {
            (Kind::Range { .. }, Value::Range(v)) => Some(self.snap(v).to_string()),
            (Kind::Toggle { .. }, Value::Toggle(on)) => Some(on.to_string()),
            (Kind::Choice { options, .. }, Value::Choice(i)) if i < options.len() => Some(i.to_string()),
            _ => None,
        }
    }

    /// `v` on the slider's nearest stop, inside its range.
    pub fn snap(&self, v: f64) -> f64 {
        let Kind::Range { min, max, step, .. } = self.kind else {
            return v;
        };
        let k = ((v - min) / step).round();
        let p = 10f64.powi(decimals(step).max(decimals(min)));
        (((min + k * step) * p).round() / p).clamp(min, max)
    }

    /// Where `v` sits along the slider, 0..1.
    pub fn fraction(&self, v: f64) -> f64 {
        match self.kind {
            Kind::Range { min, max, .. } => ((v - min) / (max - min)).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    /// The value at `f` (0..1) along the slider.
    pub fn at_fraction(&self, f: f64) -> f64 {
        match self.kind {
            Kind::Range { min, max, .. } => self.snap(min + f.clamp(0.0, 1.0) * (max - min)),
            _ => 0.0,
        }
    }

    /// How far one wheel notch or arrow press moves the slider: the step, or a round multiple
    /// of it so the whole range takes at most about a hundred presses.
    pub fn coarse_step(&self) -> f64 {
        let Kind::Range { min, max, step, .. } = self.kind else {
            return 0.0;
        };
        let stops = (max - min) / step;
        let m = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0]
            .into_iter()
            .find(|m| stops / m <= 100.0)
            .unwrap_or(1000.0);
        step * m
    }

    /// `v` moved `notches` presses (fine: by single steps).
    pub fn nudge(&self, v: f64, notches: i32, fine: bool) -> f64 {
        let Kind::Range { step, .. } = self.kind else {
            return v;
        };
        if fine || notches == 0 {
            return self.snap(v + step * f64::from(notches));
        }
        // To the next round multiple of the press in that direction, so presses land on round
        // values whatever the slider was dragged to.
        let by = self.coarse_step();
        let x = v / by;
        let grid = if notches > 0 { (x + 1e-6).floor() } else { (x - 1e-6).ceil() };
        self.snap((grid + f64::from(notches)) * by)
    }

    /// `v` written as the screen shows it.
    pub fn format(&self, value: Value) -> String {
        match (&self.kind, value) {
            (Kind::Range { unit, .. }, Value::Range(v)) => unit.format(v),
            (Kind::Toggle { .. }, Value::Toggle(on)) => (if on { "On" } else { "Off" }).to_string(),
            (Kind::Choice { options, .. }, Value::Choice(i)) => {
                options.get(i).map_or_else(String::new, |(_, label)| label.clone())
            }
            _ => String::new(),
        }
    }
}

fn range(key: &str, label: &str, help: &'static str, (min, max, step, default): (f64, f64, f64, f64), unit: Unit) -> Setting {
    Setting { key: key.to_string(), label: label.to_string(), help, kind: Kind::Range { min, max, step, default, unit } }
}

fn toggle(key: &str, label: &str, help: &'static str, default: bool) -> Setting {
    Setting { key: key.to_string(), label: label.to_string(), help, kind: Kind::Toggle { default } }
}

fn choice(key: &str, label: &str, help: &'static str, options: Vec<(String, String)>, default: usize) -> Setting {
    Setting { key: key.to_string(), label: label.to_string(), help, kind: Kind::Choice { options, default } }
}

fn build_words(p: BuildParam) -> (&'static str, &'static str, Unit) {
    use BuildParam as B;
    match p {
        B::DroneMass => ("All-up weight", "The whole drone with its battery, ready to fly.", Unit::Grams),
        B::BatteryCells => ("Battery cells", "Cells in series: more cells, more voltage and punch.", Unit::Cells),
        B::MotorKv => ("Motor KV", "Motor speed per volt. High KV spins small props fast.", Unit::Suffix(0, " KV")),
        B::Blades => ("Prop blades", "More blades grip more air, and draw more current.", Unit::Suffix(0, " blades")),
        B::PropRadius => ("Prop size", "Prop diameter, as props are sold.", Unit::DiameterInches),
        B::PropPitch => ("Prop pitch", "How far a prop would screw forward in one turn.", Unit::Inches),
        B::PropWidth => ("Blade width", "Wider blades push more air at a given speed.", Unit::Millimetres(1)),
        B::MotorWidth => ("Stator width", "Wider stators make more torque (a 2306 is 23 mm).", Unit::Millimetres(1)),
        B::MotorHeight => ("Stator height", "Taller stators make more torque (a 2306 is 6 mm).", Unit::Millimetres(1)),
        B::CameraAngle => ("Camera uptilt", "How far the camera tilts up. Fast flying wants more.", Unit::Suffix(0, "°")),
        B::Efficiency => ("Prop efficiency", "How much of the props' ideal lift the frame really gets.", Unit::Num(2)),
        B::BatteryMah => ("Battery capacity", "Bigger packs fly longer and weigh more.", Unit::Suffix(0, " mAh")),
    }
}

fn physics_words(p: PhysicsParam) -> (&'static str, &'static str, Unit) {
    use PhysicsParam as P;
    match p {
        P::AirDensity => ("Air density", "Thicker air gives more lift and more drag. Sea level: 1.225.", Unit::Suffix(3, " kg/m³")),
        P::DroneDensity => ("Frame density", "How solid the drone is, for its size against the air.", Unit::Suffix(0, " kg/m³")),
        P::DragCoefficient => ("Drag coefficient", "How hard the air holds the frame back.", Unit::Num(2)),
        P::ResistancePer1000Ft => ("Wire resistance", "Motor winding resistance: higher wastes power as heat.", Unit::Suffix(1, " Ω/kft")),
        P::TempCoeff => ("Wire temp coefficient", "How much the windings' resistance grows as they heat.", Unit::Num(5)),
        P::SpecificHeat => ("Motor specific heat", "Heat a motor soaks up before it gets hot.", Unit::Suffix(0, " J/kg·K")),
        P::HeatTransfer => ("Motor cooling", "How fast a motor sheds heat to the air.", Unit::Suffix(0, " W/K")),
    }
}

fn axis_help(axis: ControlAxis, what: &str) -> &'static str {
    match (axis, what) {
        (ControlAxis::Roll, "rc") => "How fast the drone rolls, across the whole stick.",
        (ControlAxis::Pitch, "rc") => "How fast the drone flips forward and back, across the whole stick.",
        (ControlAxis::Yaw, "rc") => "How fast the drone turns, across the whole stick.",
        (_, "super") => "Adds rate towards full stick: a calm centre and fast flips.",
        _ => "Softens the stick around centre, for fine control.",
    }
}

fn axis_name(axis: ControlAxis) -> &'static str {
    match axis {
        ControlAxis::Roll => "Roll",
        ControlAxis::Pitch => "Pitch",
        ControlAxis::Yaw => "Yaw",
        ControlAxis::Throttle => "Throttle",
    }
}

/// The rate axes (throttle has no rate).
pub const RATE_AXES: [ControlAxis; 3] = [ControlAxis::Roll, ControlAxis::Pitch, ControlAxis::Yaw];

/// An `f32` constant as the decimal it is written as (`0.7`, not `0.699999988079071`).
fn f(x: f32) -> f64 {
    x.to_string().parse().unwrap_or(f64::from(x))
}

static SETTINGS: LazyLock<Vec<Setting>> = LazyLock::new(|| {
    let mut all = vec![
        choice(
            "drone",
            "Drone",
            "The drone you fly: real models, as their makers sell them.",
            PRESETS.iter().map(|p| (format!("fpvhorizon.preset.{}", p.id), p.name.to_string())).collect(),
            0,
        ),
        toggle("custom_build", "Custom build", "Fly your own build from the sliders below instead of the drone as sold.", false),
        toggle("hide_hud", "Hide HUD and hands", "Hides the hotbar, hands and the rest of the game's HUD while a drone is out.", true),
        toggle("show_sticks", "Stick gimbals", "Shows your sticks at the bottom of the OSD, as a radio's screen does.", true),
        range("fpv_fov", "Field of view", "How much the drone's camera sees. Real FPV cameras: 120-150°.", (60.0, 150.0, 1.0, 110.0), Unit::Suffix(0, "°")),
        choice(
            "fisheye",
            "Fisheye lens",
            "Bends the picture like a wide FPV lens: the middle magnified, the edges squeezed.",
            ["off", "light", "medium", "strong"]
                .iter()
                .zip(["Off", "Light", "Medium", "Strong"])
                .map(|(id, label)| (format!("fpvhorizon.fisheye.{id}"), label.to_string()))
                .collect(),
            FISHEYE_DEFAULT,
        ),
        range("mouse_sensitivity", "Mouse sensitivity", "How far the mouse turns the drone when you fly without a gamepad.", (0.1, 5.0, 0.05, 1.0), Unit::Times),
        range("gravity", "Gravity", "How hard the world pulls the drone down. Earth: 9.8.", (4.0, 30.0, 0.1, 9.8), Unit::Suffix(1, " m/s²")),
        toggle("throttle_centre_hover", "Stick centre is hover", "A gamepad's throttle stick springs back to centre: let centre hold a hover.", true),
        toggle("mode_3d", "3D mode", "Reversible motors: the lower half of the throttle flies upside down.", false),
        range("deadzone", "Stick deadzone", "Stick movement ignored around centre, so a worn stick doesn't drift.", (0.0, 0.3, 0.01, f(c::DEFAULT_DEADZONE)), Unit::Percent),
        range("range", "Video range", "How far the video link reaches before the picture is lost and the drone disarms. All the way right: unlimited.", (32.0, RANGE_UNLIMITED, 8.0, 160.0), Unit::Range),
        toggle("load_chunks", "Load the world around the drone", "Single player: the world loads and draws around the drone instead of you, so it can fly anywhere.", true),
        toggle("battery_sim", "Battery drain and sag", "The pack drains as you fly and its voltage sags under load. Off: a full pack that never runs down.", true),
        toggle("heat_sim", "Motor heat", "Motors heat up under load and lose power when hot. Off: they stay cool.", true),
        range("motor_volume", "Motors", "How loud the motors whine.", (0.0, 1.0, 0.05, 0.6), Unit::Percent),
        range("wind_volume", "Wind", "How loud the wind rushes past at speed.", (0.0, 1.0, 0.05, 0.8), Unit::Percent),
    ];
    for axis in RATE_AXES {
        let a = axis.key();
        all.push(range(&format!("rate_{a}_rc"), "RC rate", axis_help(axis, "rc"), (f(c::RC_RATE_MIN), f(c::RC_RATE_MAX), 0.01, f(c::DEFAULT_RC_RATE)), Unit::Num(2)));
        all.push(range(&format!("rate_{a}_super"), "Super rate", axis_help(axis, "super"), (f(c::SUPER_RATE_MIN), f(c::SUPER_RATE_MAX), 0.01, f(c::DEFAULT_SUPER_RATE)), Unit::Num(2)));
        all.push(range(&format!("rate_{a}_expo"), "Expo", axis_help(axis, "expo"), (f(c::EXPO_MIN), f(c::EXPO_MAX), 0.01, f(c::DEFAULT_EXPO)), Unit::Num(2)));
    }
    for p in BuildParam::ALL {
        let (label, help, unit) = build_words(p);
        all.push(range(&format!("build_{}", p.key()), label, help, (f(p.min()), f(p.max()), f(p.step()), f(p.default_value())), unit));
    }
    for p in PhysicsParam::ALL {
        let (label, help, unit) = physics_words(p);
        all.push(range(&format!("physics_{}", p.key()), label, help, (f(p.min()), f(p.max()), f(p.step()), f(p.default_value())), unit));
    }
    all
});

/// Every setting the mod keeps, in the order Options > Mods lists them.
pub fn settings() -> &'static [Setting] {
    &SETTINGS
}

/// Setting `key`'s index in [`settings`].
pub fn index(key: &str) -> Option<usize> {
    SETTINGS.iter().position(|s| s.key == key)
}

/// Setting `key`.
#[cfg(test)]
pub fn setting(key: &str) -> &'static Setting {
    &SETTINGS[index(key).unwrap_or_else(|| panic!("no setting {key}"))]
}

/// The pages of the setup screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Drone,
    Camera,
    Rates,
    Controls,
    Link,
    Sound,
    Physics,
}

impl Page {
    pub const ALL: [Page; 7] = [Page::Drone, Page::Camera, Page::Rates, Page::Controls, Page::Link, Page::Sound, Page::Physics];

    pub fn title(self) -> &'static str {
        match self {
            Page::Drone => "DRONE",
            Page::Camera => "CAMERA",
            Page::Rates => "RATES",
            Page::Controls => "CONTROLS",
            Page::Link => "VIDEO LINK",
            Page::Sound => "SOUND",
            Page::Physics => "PHYSICS",
        }
    }

    pub fn index(self) -> usize {
        Page::ALL.iter().position(|p| *p == self).unwrap_or(0)
    }

    /// The page `by` along from this one, wrapping round.
    pub fn step(self, by: i32) -> Page {
        let n = Page::ALL.len() as i32;
        Page::ALL[(self.index() as i32 + by).rem_euclid(n) as usize]
    }
}

/// The big picture at the top of a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hero {
    /// The drone picker, with its specs and how it flies.
    Drone,
    /// The lens: a test card through the fisheye and the camera's view cone.
    Camera,
    /// The rate curves.
    Rates,
    /// The sticks live, with the deadzone.
    Sticks,
    /// The link fading with distance, and the LOW SIGNAL badge it brings.
    Link,
    /// A word of warning.
    Physics,
}

/// A buttons row's job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Copies the chosen drone into the build sliders.
    CopyPreset,
    /// Sets all three axes to a rate profile (an index into [`RATE_PROFILES`]).
    RateProfile(usize),
}

/// A row of a page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Row {
    Hero(Hero),
    Header(&'static str),
    /// A setting, by its index in [`settings`].
    Setting(usize),
    /// A row of buttons: the rate profiles.
    RateProfiles,
    /// A single button.
    Button(Action),
    /// The keys and pad buttons, for reference.
    Keys,
    /// A line of quiet words.
    Note(&'static str),
}

/// The rows of `page`; `custom` is whether Custom Build is on (its sliders show then).
pub fn rows(page: Page, custom: bool) -> Vec<Row> {
    let s = |key: &str| Row::Setting(index(key).unwrap_or_else(|| panic!("no setting {key}")));
    let build = |p: BuildParam| s(&format!("build_{}", p.key()));
    match page {
        Page::Drone => {
            let mut rows = vec![Row::Hero(Hero::Drone), s("custom_build")];
            if custom {
                use BuildParam as B;
                rows.push(Row::Button(Action::CopyPreset));
                rows.push(Row::Header("POWER"));
                rows.extend([B::BatteryCells, B::BatteryMah, B::MotorKv, B::DroneMass].map(build));
                rows.push(Row::Header("PROPS"));
                rows.extend([B::PropRadius, B::PropPitch, B::Blades, B::PropWidth, B::Efficiency].map(build));
                rows.push(Row::Header("MOTORS"));
                rows.extend([B::MotorWidth, B::MotorHeight].map(build));
                rows.push(Row::Header("CAMERA"));
                rows.push(build(B::CameraAngle));
            } else {
                rows.push(Row::Note("Turn on to tune your own: battery, motors, props, camera."));
            }
            rows
        }
        Page::Camera => vec![
            Row::Hero(Hero::Camera),
            s("fpv_fov"),
            s("fisheye"),
            Row::Header("ON SCREEN"),
            s("hide_hud"),
            s("show_sticks"),
        ],
        Page::Rates => {
            let mut rows = vec![Row::Hero(Hero::Rates), Row::RateProfiles];
            for axis in RATE_AXES {
                let a = axis.key();
                rows.push(Row::Header(match axis {
                    ControlAxis::Roll => "ROLL",
                    ControlAxis::Pitch => "PITCH",
                    _ => "YAW",
                }));
                rows.extend([s(&format!("rate_{a}_rc")), s(&format!("rate_{a}_super")), s(&format!("rate_{a}_expo"))]);
            }
            rows
        }
        Page::Controls => vec![
            Row::Hero(Hero::Sticks),
            s("deadzone"),
            s("throttle_centre_hover"),
            s("mode_3d"),
            s("mouse_sensitivity"),
            Row::Header("KEYS"),
            Row::Keys,
        ],
        Page::Link => vec![Row::Hero(Hero::Link), s("range"), s("load_chunks")],
        Page::Sound => vec![s("motor_volume"), s("wind_volume")],
        Page::Physics => {
            let mut rows = vec![
                Row::Hero(Hero::Physics),
                Row::Header("SIMULATION"),
                s("battery_sim"),
                s("heat_sim"),
                s("gravity"),
                Row::Header("AIR AND MOTORS"),
            ];
            rows.extend(PhysicsParam::ALL.map(|p| s(&format!("physics_{}", p.key()))));
            rows
        }
    }
}

/// The settings `page` holds whatever is shown (Custom Build's sliders included): what its
/// Reset puts back. The drone choice itself stays.
pub fn page_settings(page: Page) -> Vec<usize> {
    rows(page, true)
        .into_iter()
        .filter_map(|r| match r {
            Row::Setting(i) => Some(i),
            _ => None,
        })
        .collect()
}

/// The axis whose rate setting `key` is.
pub fn rate_axis(key: &str) -> Option<ControlAxis> {
    RATE_AXES.into_iter().find(|a| key.starts_with(&format!("rate_{}_", a.key())))
}

/// The axis's name, for the rate graph.
pub fn rate_axis_name(axis: ControlAxis) -> &'static str {
    axis_name(axis)
}

/// A set of rates for all three axes, from gentle to sharp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RateProfile {
    pub name: &'static str,
    pub rc: f64,
    pub super_rate: f64,
    pub expo: f64,
}

pub const RATE_PROFILES: [RateProfile; 4] = [
    RateProfile { name: "Cinematic", rc: 0.80, super_rate: 0.50, expo: 0.30 },
    RateProfile { name: "Default", rc: 1.00, super_rate: 0.70, expo: 0.00 },
    RateProfile { name: "Freestyle", rc: 1.20, super_rate: 0.73, expo: 0.20 },
    RateProfile { name: "Race", rc: 1.60, super_rate: 0.55, expo: 0.00 },
];

/// Degrees a second at stick `stick` (-1..1) with these rates.
pub fn rate_at(rc: f64, super_rate: f64, expo: f64, stick: f32) -> f32 {
    let mut cfg = ControllerConfig::default();
    let i = ControlAxis::Roll as usize;
    cfg.rc_rate[i] = rc as f32;
    cfg.super_rate[i] = super_rate as f32;
    cfg.expo[i] = expo as f32;
    cfg.rate_deg_per_sec(ControlAxis::Roll, stick)
}

/// Where a point of the fisheye preview lands on screen. `src` is a point of the camera's
/// own (straight) picture, `[-0.5, 0.5]` across each side, `aspect` the picture's width over
/// its height, `strength` the lens's. The lens shader maps the other way (screen to source:
/// `src = p (1 + S r²) / (1 + S)`, `r` 1 at the corners), so this solves it for `p` along
/// the ray through `src`.
pub fn fisheye_to_screen(src: [f32; 2], aspect: f32, strength: f32) -> [f32; 2] {
    if strength <= 0.0 {
        return src;
    }
    let corner = 0.25 * (aspect * aspect + 1.0);
    let r2 = ((src[0] * aspect).powi(2) + src[1].powi(2)) / corner;
    // 1 = l (1 + S l² r²) / (1 + S): increasing in l, so bisect it.
    let (mut lo, mut hi) = (0.0_f32, 1.0 + strength);
    for _ in 0..32 {
        let mid = (lo + hi) / 2.0;
        if mid * (1.0 + strength * mid * mid * r2) < 1.0 + strength { lo = mid } else { hi = mid }
    }
    let l = (lo + hi) / 2.0;
    [src[0] * l, src[1] * l]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_is_unique_and_valid() {
        let mut keys: Vec<&str> = settings().iter().map(|s| s.key.as_str()).collect();
        for k in &keys {
            assert!(k.len() <= 64 && k.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'), "{k}");
        }
        keys.sort_unstable();
        let n = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), n, "duplicate setting keys");
    }

    #[test]
    fn defaults_sit_on_their_sliders() {
        for s in settings() {
            if let Kind::Range { min, max, default, .. } = s.kind {
                assert!(min < max, "{}", s.key);
                assert!((min..=max).contains(&default), "{} default {default}", s.key);
                assert!((s.snap(default) - default).abs() < 1e-9, "{} default {default} is off its steps", s.key);
            }
        }
    }

    #[test]
    fn every_setting_is_on_a_page() {
        let shown: Vec<usize> = Page::ALL.iter().flat_map(|p| page_settings(*p)).collect();
        for (i, s) in settings().iter().enumerate() {
            // The drone is chosen with the picker, not a row.
            if s.key != "drone" {
                assert!(shown.contains(&i), "{} is on no page", s.key);
            }
        }
    }

    #[test]
    fn snapping_is_clean() {
        let s = setting("physics_air_density");
        assert_eq!(s.snap(1.2251), 1.225);
        assert_eq!(s.encode(Value::Range(0.25000000001)), Some("0.25".into()));
        let r = setting("range");
        assert_eq!(r.snap(1000.0), RANGE_UNLIMITED);
        assert_eq!(r.snap(-5.0), 32.0);
        assert_eq!(r.snap(165.0), 168.0);
        let v = setting("motor_volume");
        assert_eq!(v.snap(0.62), 0.6);
        assert_eq!(v.at_fraction(1.0), 1.0);
        assert_eq!(v.at_fraction(0.0), 0.0);
    }

    #[test]
    fn presses_move_by_round_amounts() {
        let kv = setting("build_motor_kv");
        assert_eq!(kv.coarse_step(), 500.0);
        assert_eq!(kv.nudge(2400.0, 1, false), 2500.0);
        assert_eq!(kv.nudge(2400.0, 1, true), 2450.0);
        let fov = setting("fpv_fov");
        assert_eq!(fov.coarse_step(), 1.0);
        assert_eq!(fov.nudge(110.0, -3, false), 107.0);
        assert_eq!(fov.nudge(150.0, 1, false), 150.0);
        let mass = setting("build_drone_mass");
        assert_eq!(mass.nudge(0.65, 1, false), 0.66);
        assert_eq!(mass.nudge(0.66, -1, false), 0.64);
        let volume = setting("motor_volume");
        assert_eq!(volume.nudge(0.6, 1, false), 0.65);
        assert_eq!(volume.nudge(0.6, -1, false), 0.55);
    }

    #[test]
    fn values_are_written_in_their_units() {
        assert_eq!(setting("fpv_fov").format(Value::Range(110.0)), "110°");
        assert_eq!(setting("build_drone_mass").format(Value::Range(0.65)), "650 g");
        assert_eq!(setting("build_prop_radius").format(Value::Range(0.0635)), "5.0 in");
        assert_eq!(setting("build_battery_cells").format(Value::Range(6.0)), "6S");
        assert_eq!(setting("deadzone").format(Value::Range(0.05)), "5%");
        assert_eq!(setting("fisheye").format(Value::Choice(2)), "Medium");
        assert_eq!(setting("mode_3d").format(Value::Toggle(true)), "On");
    }

    #[test]
    fn encoded_values_read_back() {
        assert_eq!(setting("hide_hud").encode(Value::Toggle(false)), Some("false".into()));
        assert_eq!(setting("fisheye").encode(Value::Choice(3)), Some("3".into()));
        assert_eq!(setting("fisheye").encode(Value::Choice(4)), None);
        assert_eq!(setting("fpv_fov").encode(Value::Toggle(true)), None);
    }

    #[test]
    fn default_profile_matches_the_defaults() {
        let d = RATE_PROFILES[1];
        assert_eq!(d.rc, f(c::DEFAULT_RC_RATE));
        assert_eq!(d.super_rate, f(c::DEFAULT_SUPER_RATE));
        assert_eq!(d.expo, f(c::DEFAULT_EXPO));
        let max: Vec<f32> = RATE_PROFILES.iter().map(|p| rate_at(p.rc, p.super_rate, p.expo, 1.0)).collect();
        assert!(max[0] < max[1] && max[1] < max[2], "profiles get sharper: {max:?}");
        for p in RATE_PROFILES {
            for (key, v) in [("rate_roll_rc", p.rc), ("rate_roll_super", p.super_rate), ("rate_roll_expo", p.expo)] {
                assert_eq!(setting(key).snap(v), v, "{} {key}", p.name);
            }
        }
    }

    #[test]
    fn the_top_range_stop_is_unlimited() {
        let r = setting("range");
        assert_eq!(r.at_fraction(1.0), RANGE_UNLIMITED);
        assert_eq!(r.format(Value::Range(RANGE_UNLIMITED)), "Unlimited");
        assert_eq!(r.format(Value::Range(512.0)), "512 m");
        assert_eq!(link_quality(1.0e6, RANGE_UNLIMITED), 100.0);
        assert!(link_quality(600.0, 512.0) <= 0.0);
    }

    #[test]
    fn link_fades_and_comes_back() {
        assert_eq!(link_quality(0.0, 160.0), 100.0);
        assert_eq!(link_quality(48.0, 160.0), 100.0);
        assert_eq!(link_quality(160.0, 160.0), 0.0);
        for q in [LINK_LOW, LINK_CRITICAL] {
            let d = distance_at_quality(q, 160.0);
            assert!((link_quality(d, 160.0) - q).abs() < 1e-3);
        }
    }

    #[test]
    fn fisheye_preview_inverts_the_shader() {
        let aspect = 16.0 / 9.0;
        for strength in FISHEYE_STRENGTH {
            for p in [[0.0, 0.0], [0.2, 0.1], [0.5, 0.5], [-0.4, 0.25], [0.5, 0.0]] {
                let screen = fisheye_to_screen(p, aspect, strength);
                // The shader's own map, screen to source.
                let corner = 0.25 * (aspect * aspect + 1.0);
                let r2 = ((screen[0] * aspect).powi(2) + screen[1].powi(2)) / corner;
                let k = (1.0 + strength * r2) / (1.0 + strength);
                assert!((screen[0] * k - p[0]).abs() < 1e-4 && (screen[1] * k - p[1]).abs() < 1e-4, "{strength} {p:?}");
            }
            // The corners stay on the corners.
            let c = fisheye_to_screen([0.5, 0.5], aspect, strength);
            assert!((c[0] - 0.5).abs() < 1e-4 && (c[1] - 0.5).abs() < 1e-4);
        }
    }
}
