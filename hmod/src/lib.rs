//! FPV Horizon: fly a real FPV quad in Minecraft, as in a drone sim.
//!
//! H (L1 + R1 on a pad) puts the chosen drone on the ground in front of you and takes you to its
//! camera; the pilot's body stays where it stood. Arm with R (Triangle), fly, reset with
//! Backspace (Square), switch view with V (D-pad up), switch flight mode (acro, angle, horizon,
//! 3D) with M (L1), change drone with [ ] (D-pad) on the ground, and H again lands you back in
//! your own eyes.
//!
//! J (Select on a pad while a drone is out, or the button on the pause menu) opens the setup
//! screen (`src/setup/`), where every setting is, with live previews.
//!
//! Each client tick (`client_tick_post`) runs the drone's physics against the world's blocks.
//! Each frame (`camera_pose`) turns it by the sticks and puts the camera on it. The tick module
//! (`src/adapter.rs`) reads the gamepad and the player, which the mod API has no read for.

pub mod drone;
mod font;
mod osd;

pub use osd::PadFamily;
mod render;
mod settings;
mod setup;
mod sound;

use mod_api::{
    CameraPoseEvent, CameraType, ClientHost, ClientTickPostEvent, Flow, GuiRenderPostEvent, Hooks,
    HudMode, InputKeyEvent, KeyAction, ModelBakingCompletedEvent, MovementInputEvent, Registrar,
    RegistrationContext, RenderLevelStage, RenderLevelStageEvent, ScreenKeyPressedPreEvent,
    ScreenMouseButtonPressedPreEvent, ScreenMouseButtonReleasedPreEvent, ScreenMouseDraggedPreEvent,
    ScreenMouseScrolledPreEvent, ScreenRenderPostEvent, StandaloneModelsEvent,
};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Gamepad buttons, as bits the tick module sets.
pub const PAD_L1: u32 = 1;
pub const PAD_R1: u32 = 2;
pub const PAD_ARM: u32 = 4;
pub const PAD_RESET: u32 = 8;
pub const PAD_VIEW: u32 = 16;
pub const PAD_PREV: u32 = 32;
pub const PAD_NEXT: u32 = 64;
pub const PAD_DOWN: u32 = 128;
pub const PAD_SOUTH: u32 = 256;
pub const PAD_SELECT: u32 = 512;

/// Seconds since the mod first asked, for what blinks and breathes.
pub(crate) fn time() -> f32 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    EPOCH.get_or_init(Instant::now).elapsed().as_secs_f32()
}

/// A client tick, seconds.
const TICK: f32 = 0.05;

/// The movement keys held.
#[derive(Default, Clone, Copy)]
pub struct KeysHeld {
    pub w: bool,
    pub s: bool,
    pub a: bool,
    pub d: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl KeysHeld {
    const NONE: Self = Self { w: false, s: false, a: false, d: false, up: false, down: false, left: false, right: false };

    pub fn yaw(&self) -> f32 {
        f32::from(u8::from(self.d || self.right)) - f32::from(u8::from(self.a || self.left))
    }

    pub fn throttle(&self) -> f32 {
        f32::from(u8::from(self.w || self.up)) - f32::from(u8::from(self.s || self.down))
    }
}

/// The pilot, as the tick module last saw them.
#[derive(Clone, Copy)]
pub struct PlayerView {
    pub eye: [f64; 3],
    pub feet: [f64; 3],
    pub yaw: f32,
    pub alive: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Through the drone's camera.
    Fpv,
    /// Behind and above the drone.
    Chase,
    /// From the pilot's eyes, watching it (line of sight).
    Los,
    /// Choosing a drone: the camera turns round it, close.
    Showcase,
}

impl View {
    fn next(self) -> Self {
        match self {
            View::Fpv => View::Chase,
            View::Chase => View::Los,
            View::Los | View::Showcase => View::Fpv,
        }
    }

    fn label(self) -> &'static str {
        match self {
            View::Fpv => "FPV",
            View::Chase => "CHASE",
            View::Los => "LOS",
            View::Showcase => "SELECT",
        }
    }
}

/// Key presses waiting for the next tick.
#[derive(Default)]
pub struct Requests {
    toggle: bool,
    arm: bool,
    reset: bool,
    view: bool,
    flight_mode: bool,
    preset_step: i32,
}

/// What the hooks, the mixins and the tick module share.
pub struct Shared {
    pub drone: Option<drone::DroneState>,
    /// The preset flying, and the settings page's choice last seen.
    pub preset: usize,
    preset_seen: Option<usize>,
    custom: Option<bool>,
    pub keys: KeysHeld,
    requests: Requests,
    /// The pad's sticks at the last tick, while the pad is what the player uses.
    pub sticks: Option<drone::Sticks>,
    pub pad_held: u32,
    pub pad_pressed: u32,
    /// L1 went down on its own and R1 hasn't joined it: its release switches the flight mode
    /// (with R1 it is the land chord instead).
    l1_alone: bool,
    /// When the flight mode was last switched, for the OSD's call-out.
    mode_switched: Option<Instant>,
    /// Whose button art the OSD's pad hints use, as the tick module last read it.
    pub pad_family: PadFamily,
    pub player: Option<PlayerView>,
    /// In a world, unpaused, with no screen open.
    pub playing: bool,
    view: View,
    /// On the selection turntable since then: from spawning, a new drone or a reset, until armed.
    showcase: Option<Instant>,
    /// Mouse turn since the last frame, degrees.
    mouse: [f32; 2],
    mouse_sensitivity: f32,
    last_frame: Option<Instant>,
    /// The camera's roll this frame, radians: the roll mixin applies it.
    roll: f32,
    saved_hud: Option<HudMode>,
    saved_camera: Option<CameraType>,
    show_sticks: bool,
    /// The FPV camera's field of view setting, and the field of view the world was drawn with.
    fpv_fov: f32,
    fov: f32,
    /// The FPV camera's fisheye lens: an index into `FISHEYE_EFFECTS`, 0 for none.
    fisheye: usize,
    /// Link quality 0..100 and the drone's distance from the pilot.
    link: f32,
    distance: f64,
    /// Where the link is lost, blocks from the pilot.
    range: f64,
    audio: sound::Audio,
    started: Option<Instant>,
}

pub static SHARED: Mutex<Shared> = Mutex::new(Shared {
    drone: None,
    preset: 0,
    preset_seen: None,
    custom: None,
    keys: KeysHeld::NONE,
    requests: Requests { toggle: false, arm: false, reset: false, view: false, flight_mode: false, preset_step: 0 },
    sticks: None,
    pad_held: 0,
    pad_pressed: 0,
    l1_alone: false,
    mode_switched: None,
    pad_family: PadFamily::Xbox,
    player: None,
    playing: false,
    view: View::Fpv,
    showcase: None,
    mouse: [0.0; 2],
    mouse_sensitivity: 1.0,
    last_frame: None,
    roll: 0.0,
    saved_hud: None,
    saved_camera: None,
    show_sticks: true,
    fpv_fov: 110.0,
    fov: 70.0,
    fisheye: 0,
    link: 100.0,
    distance: 0.0,
    range: 160.0,
    audio: sound::Audio::NONE,
    started: None,
});

/// The view the camera takes now: the turntable while choosing, else the pilot's choice.
fn view_now(shared: &Shared) -> View {
    if shared.showcase.is_some() { View::Showcase } else { shared.view }
}

fn flying(shared: &Shared) -> bool {
    shared.drone.as_ref().is_some_and(|d| d.active)
}

/// Whether a drone is out (the `pad` mixin keeps the pad's game actions off then).
pub fn is_flying() -> bool {
    flying(&SHARED.lock().unwrap())
}

/// The `turn` mixin: the mouse (or a pad's look stick) turned the view by `xo`, `yo` in
/// Minecraft's turn units. With a drone out the turn goes to the drone and the pilot's head
/// stays put: returns true to cancel the player's own turn.
pub fn take_turn(xo: f64, yo: f64) -> bool {
    let mut shared = SHARED.lock().unwrap();
    if !flying(&shared) {
        return false;
    }
    // A pad flies by its own sticks (rates): its look stick must not also turn the drone.
    if shared.sticks.is_none() {
        let k = shared.mouse_sensitivity;
        shared.mouse[0] += (xo * 0.15) as f32 * k;
        shared.mouse[1] += (yo * 0.15) as f32 * k;
    }
    true
}

/// The `roll` mixin: the camera's roll for this frame, radians (0 off the drone's camera).
pub fn camera_roll() -> f32 {
    let shared = SHARED.lock().unwrap();
    if flying(&shared) && view_now(&shared) == View::Fpv { shared.roll } else { 0.0 }
}

/// What the setup screen shows of the drone and the pad.
fn live() -> setup::Live {
    let shared = SHARED.lock().unwrap();
    let drone = shared.drone.as_ref().filter(|d| d.active);
    setup::Live {
        sticks: shared.sticks,
        pad: shared.sticks.is_some().then_some(shared.pad_family),
        drone_out: drone.is_some(),
        armed: drone.is_some_and(|d| d.armed),
        distance: shared.distance,
    }
}

/// The pad's bits as the setup screen's buttons.
fn pad_input(bits: u32) -> setup::PadInput {
    setup::PadInput {
        up: bits & PAD_VIEW != 0,
        down: bits & PAD_DOWN != 0,
        left: bits & PAD_PREV != 0,
        right: bits & PAD_NEXT != 0,
        select: bits & PAD_SOUTH != 0,
        reset: bits & PAD_RESET != 0,
        prev_page: bits & PAD_L1 != 0,
        next_page: bits & PAD_R1 != 0,
    }
}

/// The fisheye lens's post chains, by the settings page's choice (Off first).
const FISHEYE_EFFECTS: [Option<&str>; 4] =
    [None, Some("fpvhorizon:fisheye_light"), Some("fpvhorizon:fisheye_medium"), Some("fpvhorizon:fisheye_strong")];

/// The `fisheye` mixin: the post chain the frame goes through for the FPV camera's lens, while
/// on the drone's camera.
pub fn fisheye_effect() -> Option<&'static str> {
    let shared = SHARED.lock().unwrap();
    if flying(&shared) && view_now(&shared) == View::Fpv {
        FISHEYE_EFFECTS.get(shared.fisheye).copied().flatten()
    } else {
        None
    }
}

/// Moves a drone box (bottom centre `pos`, `half` wide each way, `height` tall) by `movement`,
/// stopped by the blocks' collision boxes. Returns the movement allowed.
fn sweep(host: &dyn ClientHost, pos: [f64; 3], movement: [f64; 3], half: f64, height: f64) -> [f64; 3] {
    let aabb = [pos[0] - half, pos[1], pos[2] - half, pos[0] + half, pos[1] + height, pos[2] + half];
    let lo = |a: usize| (aabb[a] + movement[a].min(0.0)).floor() as i32;
    let hi = |a: usize| (aabb[a + 3] + movement[a].max(0.0)).floor() as i32;
    let mut boxes = Vec::new();
    for x in lo(0)..=hi(0) {
        for y in lo(1)..=hi(1) {
            for z in lo(2)..=hi(2) {
                for b in host.block_collision([x, y, z]) {
                    let (x, y, z) = (f64::from(x), f64::from(y), f64::from(z));
                    boxes.push([b[0] + x, b[1] + y, b[2] + z, b[3] + x, b[4] + y, b[5] + z]);
                }
            }
        }
    }
    drone::collision::sweep(aabb, movement, &boxes)
}

/// Gives the drone preset `preset`'s size and toughness and `build` (the preset's or the
/// sliders'), on the ground at its launch spot again.
fn fit(drone: &mut drone::DroneState, preset: usize, build: drone::DroneBuildConfig) {
    let p = &drone::PRESETS[preset];
    drone.build = build;
    let wheelbase = build.get(drone::BuildParam::Wheelbase);
    drone.half_width = f64::from(wheelbase / 2.0 / std::f32::consts::SQRT_2 + build.get(drone::BuildParam::PropRadius));
    drone.height = f64::from(0.03 + p.prop_radius * 0.8);
    drone.crash_speed = match p.frame {
        drone::Frame::X => 9.0,
        drone::Frame::Ducted => 14.0,
    };
    if drone.active {
        drone.reset();
    }
}

/// Puts the drone down in front of the pilot: two blocks ahead (less where a wall is), then
/// down to the ground.
fn spawn(host: &dyn ClientHost, drone: &mut drone::DroneState, player: &PlayerView) {
    let (half, height) = (drone.half_width, drone.height);
    let yaw = player.yaw.to_radians();
    let start = [player.feet[0], player.feet[1] + 0.6, player.feet[2]];
    let ahead = [-f64::from(yaw.sin()) * 2.0, 0.0, f64::from(yaw.cos()) * 2.0];
    let moved = sweep(host, start, ahead, half, height);
    let at = [start[0] + moved[0], start[1], start[2] + moved[2]];
    let fall = sweep(host, at, [0.0, -8.0, 0.0], half, height);
    drone.spawn([at[0], at[1] + fall[1], at[2]], player.yaw);
}

/// Yaw and pitch (degrees, Minecraft's) looking from `from` to `to`.
fn look(from: [f64; 3], to: [f64; 3]) -> (f32, f32) {
    let (dx, dy, dz) = (to[0] - from[0], to[1] - from[1], to[2] - from[2]);
    let yaw = (-dx).atan2(dz).to_degrees() as f32;
    let pitch = -(dy.atan2((dx * dx + dz * dz).sqrt())).to_degrees() as f32;
    (yaw, pitch)
}

fn wrap_degrees(a: f32) -> f32 {
    (a + 540.0).rem_euclid(360.0) - 180.0
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        settings::register(r);
    }

    fn standalone_models(&self, _: &RegistrationContext, event: &mut StandaloneModelsEvent<'_>) {
        event.models.extend(render::MODEL_IDS.iter().map(|id| id.to_string()));
    }

    fn model_baking_completed(&self, host: &mut dyn ClientHost, _: &mut ModelBakingCompletedEvent) {
        render::resolve(&*host);
    }

    fn input_key(&self, _: &mut dyn ClientHost, event: &mut InputKeyEvent<'_>) {
        let down = match event.action {
            KeyAction::Press => true,
            KeyAction::Repeat => return,
            KeyAction::Release => false,
        };
        let mut shared = SHARED.lock().unwrap();
        if down {
            let r = &mut shared.requests;
            match event.key {
                "key.keyboard.h" => r.toggle = true,
                "key.keyboard.r" => r.arm = true,
                "key.keyboard.backspace" => r.reset = true,
                "key.keyboard.v" => r.view = true,
                "key.keyboard.m" => r.flight_mode = true,
                "key.keyboard.left.bracket" => r.preset_step -= 1,
                "key.keyboard.right.bracket" => r.preset_step += 1,
                _ => {}
            }
        }
        let keys = &mut shared.keys;
        match event.key {
            "key.keyboard.w" => keys.w = down,
            "key.keyboard.s" => keys.s = down,
            "key.keyboard.a" => keys.a = down,
            "key.keyboard.d" => keys.d = down,
            "key.keyboard.up" => keys.up = down,
            "key.keyboard.down" => keys.down = down,
            "key.keyboard.left" => keys.left = down,
            "key.keyboard.right" => keys.right = down,
            _ => {}
        }
    }

    /// The pilot stands still while flying.
    fn movement_input(&self, _: &dyn ClientHost, event: &mut MovementInputEvent) {
        if flying(&SHARED.lock().unwrap()) {
            event.forward = 0.0;
            event.strafe = 0.0;
            event.jump = false;
            event.sneak = false;
            event.forward_key = false;
            event.backward_key = false;
            event.left_key = false;
            event.right_key = false;
            event.sprint = false;
        }
    }

    /// Each tick, after the tick module read the pad and the pilot: settings, keys, physics,
    /// link, camera mode, HUD and sound.
    fn client_tick_post(&self, host: &mut dyn ClientHost, _: &mut ClientTickPostEvent) {
        // The setup screen first: its key, and the pad's buttons while it is open (the pad's
        // Select opens it, which the drone never uses).
        let (pressed, held) = {
            let mut shared = SHARED.lock().unwrap();
            (std::mem::take(&mut shared.pad_pressed), shared.pad_held)
        };
        setup::tick(host, &live(), pad_input(pressed), pad_input(held), pressed & PAD_SELECT != 0);
        // The drone holds still while the setup screen is open, and while the game is paused.
        let paused = setup::is_open(&*host) || host.is_paused();

        let mut guard = SHARED.lock().unwrap();
        let shared = &mut *guard;
        let mutable_host = host;
        let host: &dyn ClientHost = &*mutable_host;
        shared.started.get_or_insert_with(Instant::now);

        // Settings: the drone chosen on the page (or in flight), and everything else.
        let chosen = settings::chosen_preset(host);
        let custom = host.setting_bool("custom_build").unwrap_or(false);
        let mut refit = shared.drone.is_none();
        if shared.preset_seen != Some(chosen) {
            shared.preset_seen = Some(chosen);
            shared.preset = chosen;
            refit = true;
        }
        if shared.custom != Some(custom) {
            shared.custom = Some(custom);
            refit = true;
        }
        shared.mouse_sensitivity = settings::f(host, "mouse_sensitivity").unwrap_or(1.0);
        shared.show_sticks = host.setting_bool("show_sticks").unwrap_or(true);
        shared.fpv_fov = settings::f(host, "fpv_fov").unwrap_or(110.0);
        shared.fisheye = host.setting_choice("fisheye").map_or(settings::FISHEYE_DEFAULT, |i| i as usize);
        let hide_hud = host.setting_bool("hide_hud").unwrap_or(true);
        let range = f64::from(settings::f(host, "range").unwrap_or(160.0));
        let drone = shared.drone.get_or_insert_with(drone::DroneState::new);
        settings::apply(host, drone);
        if custom {
            // Sliders move freely while not flying.
            refit |= !drone.armed && settings::build(host, shared.preset) != drone.build;
        }

        // Pad presses: L1 + R1 together, and single buttons (none while the setup screen
        // has them).
        let pressed = if paused { 0 } else { pressed };
        let r = &mut shared.requests;
        if pressed & (PAD_L1 | PAD_R1) != 0 && shared.pad_held & (PAD_L1 | PAD_R1) == PAD_L1 | PAD_R1 {
            r.toggle = true;
        }
        r.arm |= pressed & PAD_ARM != 0;
        r.reset |= pressed & PAD_RESET != 0;
        r.view |= pressed & PAD_VIEW != 0;
        // L1 alone switches the flight mode, on release, as Liftoff's mode switch does; L1 + R1
        // is the land chord, so R1 joining in at any point cancels the switch.
        let held = shared.pad_held;
        if paused {
            shared.l1_alone = false;
        } else {
            if pressed & PAD_L1 != 0 {
                shared.l1_alone = held & PAD_R1 == 0;
            }
            if held & PAD_R1 != 0 {
                shared.l1_alone = false;
            }
            if shared.l1_alone && held & PAD_L1 == 0 {
                shared.l1_alone = false;
                r.flight_mode = true;
            }
        }
        r.preset_step += i32::from(pressed & PAD_NEXT != 0) - i32::from(pressed & PAD_PREV != 0);
        let requests = std::mem::take(&mut shared.requests);
        let playing = shared.playing;

        if requests.preset_step != 0 && playing && !drone.armed {
            let n = drone::PRESETS.len() as i32;
            shared.preset = (shared.preset as i32 + requests.preset_step).rem_euclid(n) as usize;
            shared.preset_seen = Some(shared.preset);
            settings::choose_preset(host, shared.preset);
            refit = true;
            if drone.active {
                shared.showcase = Some(Instant::now());
            }
        }
        if refit && !drone.armed {
            fit(drone, shared.preset, settings::build(host, shared.preset));
        }

        match shared.player {
            Some(player) if player.alive => {
                if requests.toggle && playing {
                    if drone.active {
                        drone.despawn();
                    } else {
                        spawn(host, drone, &player);
                        shared.view = View::Fpv;
                        shared.showcase = Some(Instant::now());
                        shared.last_frame = None;
                    }
                }
            }
            _ => drone.despawn(),
        }

        if drone.active && playing {
            if requests.arm {
                if drone.armed {
                    drone.disarm();
                } else if drone.arm() {
                    shared.showcase = None;
                }
            }
            if requests.reset {
                drone.reset();
                shared.showcase = Some(Instant::now());
            }
            if requests.view {
                shared.showcase = None;
                shared.view = shared.view.next();
            }
            if requests.flight_mode {
                let mode = drone.flight_mode().next();
                drone.controller_config.flight_mode = mode;
                settings::choose_flight_mode(host, mode);
                shared.mode_switched = Some(Instant::now());
            }
        }

        // The video link fades with distance from the pilot; past the range it is lost and the
        // flight controller's failsafe disarms. Full strength out to a share of the range, then
        // a straight line down to nothing, so the warning comes early and steadily.
        if let (true, Some(player)) = (drone.active, shared.player) {
            let d = std::array::from_fn::<f64, 3, _>(|i| drone.position[i] - player.eye[i]);
            shared.distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            shared.range = range;
            shared.link = setup::model::link_quality(shared.distance, range);
            if shared.link <= 0.0 && drone.armed {
                drone.disarm();
            }
        }

        if drone.active && shared.player.is_some() && !paused {
            let (half, height) = (drone.half_width, drone.height);
            drone.physics_tick(TICK, |pos, movement| sweep(host, pos, movement, half, height));
        }

        // Off the pilot's eyes: third person, so the world draws the pilot's body.
        let active = drone.active;
        if active {
            // The fields directly: `drone` still borrows `shared`.
            let los = shared.showcase.is_none() && shared.view == View::Los;
            let wanted = if los { CameraType::FirstPerson } else { CameraType::ThirdPersonBack };
            shared.saved_camera.get_or_insert_with(|| host.camera_type());
            if host.camera_type() != wanted {
                host.set_camera_type(wanted);
            }
        } else if let Some(camera) = shared.saved_camera.take() {
            host.set_camera_type(camera);
        }
        // Hide the hotbar, the hands and the rest of the HUD while a drone is out.
        let hud = if active && hide_hud && shared.saved_hud.is_none() {
            shared.saved_hud = Some(host.hud_mode());
            Some(HudMode::AllHidden)
        } else if !(active && hide_hud) {
            shared.saved_hud.take()
        } else {
            None
        };

        if !drone.active {
            shared.showcase = None;
        }
        let far = match view_now(shared) {
            View::Los => 1.0 / (1.0 + shared.distance as f32 / 6.0),
            View::Showcase => 0.5,
            _ => 1.0,
        };
        let volumes = (
            settings::f(host, "motor_volume").unwrap_or(0.6),
            settings::f(host, "wind_volume").unwrap_or(0.8),
        );
        let state = shared.drone.as_ref().filter(|d| d.active && !paused);
        shared.audio.update(host, state, far, volumes);
        if let Some(mode) = hud {
            mutable_host.set_hud_mode(mode);
        }
    }

    /// Every frame: fly the drone by the sticks and put the camera where the view says.
    fn camera_pose(&self, host: &dyn ClientHost, event: &mut CameraPoseEvent<'_>) {
        let now = Instant::now();
        let mut guard = SHARED.lock().unwrap();
        let shared = &mut *guard;
        let dt = shared.last_frame.replace(now).map_or(0.0, |last| now.duration_since(last).as_secs_f32());
        let mouse = std::mem::take(&mut shared.mouse);
        let playing = host.playing();
        let input = drone::FrameInput {
            mouse_dx: mouse[0],
            mouse_dy: mouse[1],
            key_yaw: if playing { shared.keys.yaw() } else { 0.0 },
            key_throttle: if playing { shared.keys.throttle() } else { 0.0 },
            sticks: if playing { shared.sticks } else { None },
        };
        let view = view_now(shared);
        let showcase_time = shared.showcase.map_or(0.0, |t| t.elapsed().as_secs_f32());
        let player = shared.player;
        let Some(drone) = shared.drone.as_mut().filter(|d| d.active) else {
            shared.roll = 0.0;
            return;
        };
        if !host.is_paused() {
            // The flight controller keeps flying behind chat and the setup screen, on centred
            // sticks and the throttle where it was.
            let flying = playing && !setup::is_open(host);
            drone.frame(dt, &if flying { input } else { drone::FrameInput::default() });
        }
        let partial = event.partial;
        let at = drone.position_at(partial);
        match view {
            View::Fpv => {
                let (yaw, pitch, roll) = drone.camera_angles();
                let up = drone.orientation.transform_vec3(drone::Vec3f::UP);
                let lift = drone.height * 0.6;
                for i in 0..3 {
                    event.position[i] = at[i] + [up.x, up.y, up.z][i] as f64 * lift;
                }
                event.yaw = yaw;
                event.pitch = pitch;
                shared.roll = roll;
            }
            View::Chase => {
                let heading = drone.heading().to_radians();
                let forward = [-f64::from(heading.sin()), 0.0, f64::from(heading.cos())];
                let target = [at[0], at[1] + drone.height / 2.0, at[2]];
                let back = 0.8 + drone.half_width * 4.0;
                let wanted = [-forward[0] * back, back * 0.35, -forward[2] * back];
                // Pull the camera in where a block is in the way, as vanilla's third person does.
                let free = sweep(host, [target[0], target[1] - 0.1, target[2]], wanted, 0.1, 0.2);
                let eye = [target[0] + free[0] * 0.9, target[1] + free[1] * 0.9, target[2] + free[2] * 0.9];
                for i in 0..3 {
                    event.position[i] = eye[i];
                }
                let (yaw, pitch) = look(eye, target);
                event.yaw = yaw;
                event.pitch = pitch;
                shared.roll = 0.0;
            }
            View::Showcase => {
                // A turntable: round the drone, starting off its front left, a little above.
                let target = [at[0], at[1] + drone.height * 0.5, at[2]];
                let heading = f64::from(drone.heading().to_radians());
                let angle = heading + 0.6 + f64::from(showcase_time) * 0.35;
                let dist = 0.25 + drone.half_width * 3.2;
                let wanted = [-angle.sin() * dist, dist * 0.45, angle.cos() * dist];
                let free = sweep(host, [target[0], target[1] - 0.05, target[2]], wanted, 0.05, 0.1);
                let eye = [target[0] + free[0] * 0.9, target[1] + free[1] * 0.9, target[2] + free[2] * 0.9];
                for i in 0..3 {
                    event.position[i] = eye[i];
                }
                let (yaw, pitch) = look(eye, target);
                event.yaw = yaw;
                event.pitch = pitch;
                shared.roll = 0.0;
            }
            View::Los => {
                let eye = player.map_or([event.position[0], event.position[1], event.position[2]], |p| p.eye);
                for i in 0..3 {
                    event.position[i] = eye[i];
                }
                let (yaw, pitch) = look(eye, [at[0], at[1] + drone.height / 2.0, at[2]]);
                event.yaw = yaw;
                event.pitch = pitch;
                shared.roll = 0.0;
            }
        }
    }

    /// A real FPV camera is wide: its field of view while on the drone's camera.
    fn compute_fov(&self, _: &dyn ClientHost, event: &mut mod_api::ComputeFovEvent) {
        let mut shared = SHARED.lock().unwrap();
        if flying(&shared) {
            match view_now(&shared) {
                View::Fpv => event.degrees = shared.fpv_fov,
                // A long lens for the product shot.
                View::Showcase => event.degrees = 45.0,
                _ => {}
            }
        }
        shared.fov = event.degrees;
    }

    /// The drone, seen from outside (chase and line of sight).
    fn render_level_stage(&self, _: &dyn ClientHost, event: &mut RenderLevelStageEvent<'_>) {
        if event.stage != RenderLevelStage::AfterTranslucentBlocks {
            return;
        }
        let shared = SHARED.lock().unwrap();
        if view_now(&shared) == View::Fpv {
            return;
        }
        if let Some(drone) = shared.drone.as_ref().filter(|d| d.active) {
            render::draw(&mut *event.draw, drone, &drone::PRESETS[shared.preset], event.view.partial_tick);
        }
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        // The OSD is drawn over the screens: none under the setup screen or the pause menu.
        if setup::is_open(host) || host.is_paused() {
            return;
        }
        let shared = SHARED.lock().unwrap();
        if event.gui_hidden && shared.saved_hud.is_none() {
            return;
        }
        let Some(drone) = shared.drone.as_ref().filter(|d| d.active) else {
            return;
        };
        let to_home = look(drone.position, drone.launch).0;
        let (_, pitch, roll) = drone.camera_angles();
        let osd = osd::Osd {
            state: drone,
            preset: &drone::PRESETS[shared.preset],
            custom: host.setting_bool("custom_build").unwrap_or(false),
            horizon: (view_now(&shared) == View::Fpv).then_some((pitch, roll)),
            link: shared.link,
            distance: shared.distance,
            range: shared.range,
            home_bearing: wrap_degrees(to_home - drone.heading()),
            view: view_now(&shared).label(),
            showcase: shared.showcase.is_some(),
            index: (shared.preset, drone::PRESETS.len()),
            fov: shared.fov,
            show_sticks: shared.show_sticks,
            pad: shared.sticks.is_some().then_some(shared.pad_family),
            time: shared.started.map_or(0.0, |s| s.elapsed().as_secs_f32()),
            setup_key: &settings::setup_key_name(host),
            mode_switched: shared.mode_switched.map(|t| t.elapsed().as_secs_f32()),
        };
        osd::draw(&mut *event.batch, &osd);
    }

    /// The setup screen, and its button on the pause menu.
    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        if setup::is_open(host) {
            setup::draw(&mut *event.draw, host, &live(), event.screen.mouse);
        } else if event.screen.id == "paused" && host.mod_screen().is_none() {
            setup::draw_pause_button(&mut *event.draw, event.screen.mouse);
        }
    }

    fn screen_mouse_button_pressed_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseButtonPressedPreEvent<'_>) -> Flow {
        if setup::is_open(host) {
            setup::mouse_pressed(host, &live(), event.screen.mouse, event.button);
            return Flow::Cancel;
        }
        if event.screen.id == "paused" && event.button == 0 && setup::pause_click(event.screen.mouse) {
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_mouse_button_released_pre(&self, host: &dyn ClientHost, _: &mut ScreenMouseButtonReleasedPreEvent<'_>) -> Flow {
        if setup::is_open(host) {
            setup::mouse_released();
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_mouse_dragged_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseDraggedPreEvent<'_>) -> Flow {
        if setup::is_open(host) {
            setup::mouse_dragged(host, event.screen.mouse);
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_mouse_scrolled_pre(&self, host: &dyn ClientHost, event: &mut ScreenMouseScrolledPreEvent<'_>) -> Flow {
        if setup::is_open(host) {
            setup::mouse_scrolled(host, event.screen.mouse, event.scroll[1], event.modifiers);
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_key_pressed_pre(&self, host: &dyn ClientHost, event: &mut ScreenKeyPressedPreEvent<'_>) -> Flow {
        if setup::is_open(host) && setup::key_pressed(host, &live(), event.key, event.modifiers) {
            return Flow::Cancel;
        }
        Flow::Continue
    }
}
