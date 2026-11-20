// Game-agnostic core logic for the FPV Horizon mod.
// Handles 3D vector/quaternion math, aerodynamics, motor physics, collision response,
// Betaflight rates, stick shaping, HUD geometry, and drone state lifecycle.

pub mod build;
pub mod collision;
pub mod controller;
pub mod hud;
pub mod math;
pub mod physics;
pub mod physics_settings;
pub mod presets;
pub mod state;

pub use build::{BuildParam, DroneBuildConfig};
pub use collision::CollisionResult;
pub use controller::{ControlAxis, ControllerConfig, FlightMode, VirtualStick};
pub use hud::{GimbalHud, HudPainter};
pub use math::{Quat, Vec3f};
pub use physics::AdvancedPhysicsCore;
pub use physics_settings::{PhysicsConfig, PhysicsParam};
pub use presets::{Frame, Preset, PRESETS};
pub use state::{DroneState, FrameInput, Sticks};
