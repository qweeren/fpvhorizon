# FPV Drone for Horizon (Rust)

A native Horizon port of the FPV Drone mod. Enables realistic quadcopter physics, Betaflight-style rates, and acro flight mechanics inside Horizon.

## Features

- **Advanced Quadcopter Physics Engine**:
  - Full 6-DoF aerodynamic simulation.
  - Blade element theory with multi-section propeller lift and drag curves.
  - Electric motor dynamics with back-EMF, winding resistance, and thermal modeling (Joule heating and convective cooling).
  - Battery voltage sag curves under throttle.
  - Ground collision resolution with glancing angles and elastic bouncing.
- **Betaflight Rate System**:
  - RC Rate, Super Rate, and Expo for Roll, Pitch, and Yaw.
  - Linear stick shaping with configurable deadzones.
  - 3D mode (reversible throttle) option.
- **On-Screen Gimbal HUD**:
  - Dual gimbal stick position indicators (Throttle/Yaw on left, Roll/Pitch on right).
  - Visual crosshairs with real-time marker tracking.
- **Native Horizon Mixin Architecture**:
  - Pure, game-agnostic Rust core in `core/` with 100% unit-tested math and physics.
  - Clean Horizon adapter in `mixins/fpvdrone/` hooking into `native_tick`, `native_hud`, `native_key`, `native_movement_input`, and `native_settings`.

## Controls

| Key | Action |
|---|---|
| **H** | Toggle FPV Drone Flight Mode |
| **Mouse X / Y** | Roll & Pitch (Acro mode control) |
| **W / S** (or **Up / Down**) | Throttle Up / Down |
| **A / D** (or **Left / Right**) | Yaw Left / Right |

## Settings (Options > Mods > FPV Drone)

- **Drone Mode**: Toggle flight on/off.
- **3D Mode**: Toggle between standard throttle (0% to 100%) and 3D acrobatic throttle (-100% to +100%).
- **Camera Angle**: Cycle camera up-tilt angle (10°, 20°, 30°, 45°).
- **Reset Drone**: Restore default build and physics parameters.

## Building & Packaging

### Running Unit Tests

Run the standalone core test suite (requires no game files or running client):

```bash
cargo test --lib
```

### Packaging `.hnmod`

To package the bundle into `dist/fpvdrone.hnmod`:

```bash
python tools/package.py
```

To package and automatically install to Horizon's native mods folder:

```bash
python tools/package.py --install
```
