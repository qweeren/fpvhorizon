// Adapter connecting the pure Rust FPV drone core to Horizon's game instance.
// Exposes tick, HUD, key, movement, settings, and network hooks.

use crate::game::fpvdrone as core;
use crate::game::Game;
use std::sync::Mutex;
use std::time::Instant;

static STATE: Mutex<Option<core::DroneState>> = Mutex::new(None);
static KEYS_HELD: Mutex<KeysHeld> = Mutex::new(KeysHeld::new());
static START_TIME: Mutex<Option<Instant>> = Mutex::new(None);

#[derive(Default, Clone, Copy)]
struct KeysHeld {
    w: bool,
    s: bool,
    a: bool,
    d: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl KeysHeld {
    const fn new() -> Self {
        Self {
            w: false,
            s: false,
            a: false,
            d: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

fn now_nanos() -> u64 {
    let mut start = START_TIME.lock().unwrap();
    let inst = start.get_or_insert_with(Instant::now);
    inst.elapsed().as_nanos() as u64
}

fn with_drone<R, F: FnOnce(&mut core::DroneState) -> R>(f: F) -> Option<R> {
    let mut lock = STATE.lock().unwrap();
    let state = lock.get_or_insert_with(core::DroneState::new);
    Some(f(state))
}

pub fn is_active() -> bool {
    STATE.lock().unwrap().as_ref().is_some_and(|s| s.active)
}

pub fn toggle_drone(game: &mut Game<'_>) {
    let mut lock = STATE.lock().unwrap();
    let state = lock.get_or_insert_with(core::DroneState::new);
    let now = now_nanos();

    if state.active {
        let (y, p) = state.exit();
        game.yaw = y;
        game.pitch = p;
        if let Some(player) = game.player.as_mut() {
            player.flying = false;
        }
    } else {
        let pos = game.native_player().map_or([0.0; 3], |p| [p.x, p.y, p.z]);
        state.enter(pos, game.yaw, game.pitch, now);
        if let Some(player) = game.player.as_mut() {
            player.flying = true;
            player.may_fly = true;
        }
    }
}

pub fn tick(game: &mut Game<'_>) {
    let mut lock = STATE.lock().unwrap();
    let Some(state) = lock.as_mut() else { return };
    if !state.active {
        return;
    }

    let now = now_nanos();
    let dt = if state.last_nanos > 0 {
        ((now - state.last_nanos) as f32) * 1e-9
    } else {
        0.05
    };
    state.last_nanos = now;

    // Head pinning & mouse look calculation
    let mouse_dx = game.yaw - state.locked_yaw;
    let mouse_dy = game.pitch;
    game.yaw = state.locked_yaw;
    game.pitch = 0.0;

    // Movement inputs (WASD or Arrow keys)
    let keys = *KEYS_HELD.lock().unwrap();
    let yaw_input = (if keys.d || keys.right { 1.0 } else { 0.0 })
        - (if keys.a || keys.left { 1.0 } else { 0.0 });
    let throttle_input = (if keys.w || keys.up { 1.0 } else { 0.0 })
        - (if keys.s || keys.down { 1.0 } else { 0.0 });

    // Step physics & movement
    state.step_input_and_physics(dt, mouse_dx, mouse_dy, yaw_input, throttle_input, |req| {
        // Move player through world
        if let Some(p) = game.player.as_mut() {
            p.pos.x += req[0];
            p.pos.y += req[1];
            p.pos.z += req[2];
        }
        req
    });

    // Update player velocity and maintain flight abilities
    if let Some(p) = game.player.as_mut() {
        p.velocity = horizon_core::geometry::Vec3::new(
            state.velocity.x as f64,
            state.velocity.y as f64,
            state.velocity.z as f64,
        );
        p.flying = true;
        p.may_fly = true;
    }

    // Set camera rotation
    let [cam_y, cam_p, _] = state.get_camera_orientation().to_euler_angles_yxz();
    game.yaw = -cam_y.to_degrees();
    game.pitch = -cam_p.to_degrees();
}

pub fn hud(game: &mut Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    let mut lock = STATE.lock().unwrap();
    let Some(state) = lock.as_mut() else { return };
    if !state.active {
        return;
    }

    struct Painter<'a, 'g> {
        game: &'a mut Game<'g>,
        batch: &'a mut horizon_client::hud::HudBatch,
    }

    impl core::HudPainter for Painter<'_, '_> {
        fn fill_rect(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, argb: u32) {
            let w = (max_x - min_x) as f32;
            let h = (max_y - min_y) as f32;
            self.game.native_fill(self.batch, [min_x as f32, min_y as f32, w, h], argb);
        }
    }

    let mut painter = Painter { game, batch };
    let hud = core::GimbalHud::new(state.hud_yaw, state.hud_throttle, state.hud_roll, state.hud_pitch);
    hud.render(&mut painter, batch.width, batch.height);
}

pub fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if key == "key.keyboard.h" && pressed {
        toggle_drone(game);
        return;
    }

    let mut keys = KEYS_HELD.lock().unwrap();
    match key {
        "key.keyboard.w" => keys.w = pressed,
        "key.keyboard.s" => keys.s = pressed,
        "key.keyboard.a" => keys.a = pressed,
        "key.keyboard.d" => keys.d = pressed,
        "key.keyboard.up" => keys.up = pressed,
        "key.keyboard.down" => keys.down = pressed,
        "key.keyboard.left" => keys.left = pressed,
        "key.keyboard.right" => keys.right = pressed,
        _ => {}
    }
}

pub fn movement(_game: &Game<'_>, forward: &mut f32, strafe: &mut f32, jump: &mut bool, sneak: &mut bool) {
    if is_active() {
        *forward = 0.0;
        *strafe = 0.0;
        *jump = false;
        *sneak = false;
    }
}

pub fn list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "fpvdrone".into(),
        title: "FPV Drone 1.0.0".into(),
        description: "Fly as an FPV drone in Minecraft with realistic quadcopter aerodynamics.".into(),
        keys: vec!["key.keyboard.h".into()],
    });
}

pub fn settings(_game: &mut Game<'_>, buttons: &mut Vec<String>) {
    with_drone(|s| {
        buttons.push(format!("Drone Mode: {}", if s.active { "ON" } else { "OFF" }));
        buttons.push(format!("3D Mode (3D Throttle): {}", if s.controller_config.mode_3d { "ON" } else { "OFF" }));
        buttons.push(format!("Camera Angle: {:.0}°", s.build.get(core::BuildParam::CameraAngle)));
        buttons.push("Reset Drone to Defaults".to_string());
    });
}

pub fn click(game: &mut Game<'_>, button: usize) {
    with_drone(|s| match button {
        0 => toggle_drone(game),
        1 => s.controller_config.mode_3d = !s.controller_config.mode_3d,
        2 => {
            let curr = s.build.get(core::BuildParam::CameraAngle);
            let next = match curr as i32 {
                0..=14 => 20.0,
                15..=24 => 30.0,
                25..=34 => 45.0,
                _ => 10.0,
            };
            s.build.set(core::BuildParam::CameraAngle, next);
        }
        3 => {
            s.build.reset();
            s.physics_config.reset();
            s.controller_config = core::ControllerConfig::default();
        }
        _ => {}
    });
}

pub fn packet(_game: &mut Game<'_>, _packet: &horizon_client::mod_api::ClientboundPlayPacket) {
    // Custom payload handling for orientation / drone sync when multiplayer packet arrives
}
