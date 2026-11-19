//! The drone in the world: its frame model (an X quad or a ducted whoop) and its props, each
//! blade a model turned by its motor, or a blur disc once the motor spins too fast to see.
//! Models are Java block models (tools/drone_assets.py) whose unit is a centimetre.

use crate::drone::{DroneState, Frame, Preset, Quat, Vec3f};
use mod_api::{
    ClientHost, Identifier, LightCoords, ModelInstance, ModelRef, Overlay, WorldDepth, WorldDrawContext,
    WorldQuad,
};
use std::sync::Mutex;

const X_FRAME: &str = "fpvdrone:drone/x_frame";
const DUCTED: &str = "fpvdrone:drone/ducted";
const PROP: &str = "fpvdrone:drone/prop";
pub const MODEL_IDS: [&str; 3] = [X_FRAME, DUCTED, PROP];

/// A model unit is 1/16 block once baked; the models are drawn in centimetres (0.01 block).
const CM: f32 = 16.0 / 100.0;
/// The models' own sizes: the wheelbase they are drawn at and their prop's radius, metres.
const X_WHEELBASE: f32 = 0.245;
const DUCTED_WHEELBASE: f32 = 0.142;
const MODEL_PROP_RADIUS: f32 = 0.0635;
/// Height of the motor tops, model units.
const X_MOTOR_TOP: f32 = 2.1;
const DUCTED_MOTOR_TOP: f32 = 1.8;
/// Above this motor speed (rad/s) a prop is drawn as a blur disc.
const BLUR_SPEED: f32 = 120.0;

#[derive(Default)]
pub struct Models {
    x_frame: Option<ModelRef>,
    ducted: Option<ModelRef>,
    prop: Option<ModelRef>,
}

pub static MODELS: Mutex<Models> = Mutex::new(Models { x_frame: None, ducted: None, prop: None });

/// After each model bake: look the drone's models up once.
pub fn resolve(host: &dyn ClientHost) {
    let assets = host.render_assets();
    let get = |id: &str| Identifier::parse(id).ok().and_then(|id| assets.model(&id).ok());
    let mut models = MODELS.lock().unwrap();
    models.x_frame = get(X_FRAME);
    models.ducted = get(DUCTED);
    models.prop = get(PROP);
}

/// Row-major rotation matrix of `q`.
fn matrix(q: Quat) -> [[f32; 3]; 3] {
    let Quat { x, y, z, w } = q.normalize();
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w)],
        [2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w)],
        [2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

fn apply(m: &[[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

/// The instance drawing a model's local point `p` (baked, in blocks) at
/// `origin + offset + rotation * scale * (p - centre)`, the centre being the model's bottom
/// centre (0.5, 0, 0.5).
fn instance(origin: [f64; 3], offset: [f32; 3], rotation: &[[f32; 3]; 3], scale: f32) -> ModelInstance {
    let m: [[f32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| rotation[i][j] * scale));
    let centre = apply(&m, [0.5, 0.0, 0.5]);
    let t: [f32; 3] = std::array::from_fn(|i| offset[i] - centre[i]);
    let transform = [
        m[0][0], m[0][1], m[0][2], t[0],
        m[1][0], m[1][1], m[1][2], t[1],
        m[2][0], m[2][1], m[2][2], t[2],
    ];
    ModelInstance::new(origin, transform, 0xFFFF_FFFF, LightCoords::FULL_BRIGHT, Overlay::NONE)
}

/// The drone's motors in its own frame (blocks, +x right, +y up, -z forward), front right first
/// and round, matching the physics' motor order.
fn motor_positions(preset: &Preset) -> [[f32; 3]; 4] {
    let (wheelbase, top) = match preset.frame {
        Frame::X => (X_WHEELBASE, X_MOTOR_TOP),
        Frame::Ducted => (DUCTED_WHEELBASE, DUCTED_MOTOR_TOP),
    };
    let k = preset.wheelbase / 2.0 / std::f32::consts::SQRT_2;
    let y = top * 0.01 * preset.wheelbase / wheelbase;
    [[k, y, -k], [k, y, k], [-k, y, k], [-k, y, -k]]
}

pub fn draw(draw: &mut dyn WorldDrawContext, state: &DroneState, preset: &Preset, partial: f32) {
    let models = MODELS.lock().unwrap();
    let origin = state.position_at(partial);
    let rotation = matrix(state.orientation);
    let (frame, wheelbase) = match preset.frame {
        Frame::X => (models.x_frame, X_WHEELBASE),
        Frame::Ducted => (models.ducted, DUCTED_WHEELBASE),
    };
    if let Some(frame) = frame {
        let _ = draw.draw_model_instances(frame, &[instance(origin, [0.0; 3], &rotation, CM * preset.wheelbase / wheelbase)]);
    }

    let up = apply(&rotation, [0.0, 1.0, 0.0]);
    let prop_scale = CM * preset.prop_radius / MODEL_PROP_RADIUS;
    let mut blades = Vec::new();
    let mut discs = Vec::new();
    for (i, motor) in motor_positions(preset).into_iter().enumerate() {
        let at = apply(&rotation, motor);
        let speed = state.physics.motor_vel[i].abs();
        if speed > BLUR_SPEED {
            let alpha = (0x28 as f32 + 0x30 as f32 * (speed / 1500.0).min(1.0)) as u32;
            disc(&mut discs, origin, at, up, &rotation, preset.prop_radius, (alpha << 24) | 0x00D8_DCE0);
            continue;
        }
        let spin = state.physics.motor_pos[i] * if i % 2 == 0 { -1.0 } else { 1.0 };
        let count = preset.blades.round().max(2.0) as usize;
        for b in 0..count {
            let angle = spin + std::f32::consts::TAU * b as f32 / count as f32;
            let turned = Quat::from_axis_angle(Vec3f::UP, angle);
            let blade = matrix(state.orientation.mul(turned));
            blades.push(instance(origin, at, &blade, prop_scale));
        }
    }
    if let (Some(prop), false) = (models.prop, blades.is_empty()) {
        let _ = draw.draw_model_instances(prop, &blades);
    }
    if !discs.is_empty() {
        let _ = draw.quads(&discs);
    }
}

/// A translucent ring of quads where a spinning prop sweeps.
fn disc(out: &mut Vec<WorldQuad>, origin: [f64; 3], centre: [f32; 3], up: [f32; 3], rotation: &[[f32; 3]; 3], radius: f32, color: u32) {
    const SEGMENTS: usize = 16;
    let lift: [f32; 3] = std::array::from_fn(|i| centre[i] + up[i] * 0.003);
    let point = |k: usize, r: f32| -> [f64; 3] {
        let a = std::f32::consts::TAU * k as f32 / SEGMENTS as f32;
        let local = apply(rotation, [a.cos() * r, 0.0, a.sin() * r]);
        std::array::from_fn(|i| origin[i] + f64::from(lift[i] + local[i]))
    };
    let inner = radius * 0.18;
    for k in 0..SEGMENTS {
        out.push(WorldQuad::new(
            [point(k, inner), point(k, radius), point(k + 1, radius), point(k + 1, inner)],
            color,
            WorldDepth::Tested,
        ));
    }
}
