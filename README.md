# FPV Horizon

FPV Horizon is a native Horizon mod for flying FPV drones. It brings realistic quadcopter physics, Betaflight-style rates, and acro flight mechanics inside Horizon. It also ships as a `.smod` for [Sunrise](https://github.com/sunrise-org/sunrise), built from the same code.

## Features

- **Quadcopter physics**:
  - Full 6-DoF aerodynamic simulation.
  - Blade element theory with multi-section propeller lift and drag curves.
  - Electric motor dynamics with back-EMF, winding resistance, and thermal modelling (Joule heating and convective cooling).
  - Battery voltage sag under throttle.
  - A simulated flight controller: a rate PID per axis (with TPA, D setpoint weight, I-term relax and air mode) drives
    each motor through a quad-X mix, so the drone turns by its motors' thrust and prop torque against its own inertia,
    with real overshoot, motor lag and prop wash.
  - Block collisions with glancing angles and elastic bouncing.
- **Real drone presets**: 5" freestyle and race quads, a 3.5" cinewhoop, a 75mm whoop and a 7.5" long-range build, or your own custom build.
- **Betaflight rates**: RC Rate, Super Rate and Expo for roll, pitch and yaw, configurable deadzone.
- **Flight modes**: Acro, Angle (self-levelling), Horizon (levels near centre stick, flips at full stick) and 3D
  (reversible motors), switched in flight with L1 or M as in Liftoff, with an adjustable angle limit and level strength.
- **FPV view**: the camera rolls with the drone, with an adjustable field of view, an OSD and stick gimbal overlays.
- **Gamepad support**: fly with a controller's sticks, or with mouse and keyboard.
- **Motor and wind sound**.

## Controls

| Keyboard | Gamepad | Action |
|---|---|---|
| **H** | **L1 + R1** | Put the drone down and switch to its camera / return to your own eyes |
| **R** | **Triangle** | Arm / disarm |
| **V** | **D-pad up** | Switch view |
| **M** | **L1** | Switch flight mode: Acro → Angle → Horizon → 3D |
| **[** / **]** | **D-pad left / right** | Change drone (on the ground) |
| **Backspace** | **Square** | Reset the drone |
| **J** | **Select** (drone out) | Open the setup screen |
| **Mouse X / Y** | Right stick | Roll and pitch |
| **W / S** (or **Up / Down**) | Left stick up / down | Throttle |
| **A / D** (or **Left / Right**) | Left stick left / right | Yaw |

## Setup screen

Press **J** (rebindable in Controls), **Select** on a gamepad while a drone is out, or the **FPV Horizon Setup**
button on the pause menu. Every setting is there, drawn in the OSD's style, and each change applies at once. A drone in
the air holds still while it is open.

- **Drone**: a picker for the real drones with their specs, thrust-to-weight and hover throttle, and **Custom Build**
  for your own battery, motors, props and camera uptilt (it starts from the chosen drone).
- **Camera**: field of view and fisheye lens, with a test card seen through the lens and the camera's view cone.
- **Rates**: RC Rate, Super Rate and Expo per axis, live rate curves, and Cinematic / Default / Freestyle / Race
  profiles.
- **Controls**: deadzone with your sticks shown live, stick-centre hover, flight mode, angle limit and level strength, mouse sensitivity, and every key.
- **Video link**: the range, with a preview of where the LOW SIGNAL and TURN BACK warnings come.
- **Sound** and **Physics**: volumes, gravity, air density, drag and motor thermal constants.

Drag or scroll a slider, right-click any row for its default, Tab for the next page, Esc to close. The keyboard's
arrows and a pad's D-pad, A, LB/RB and B work too. The same settings are also on the mod's page in Options > Mods.

## Project layout

- `hmod/`: the Horizon mod: `horizon-mod.toml`, the mod crate in `src/`, its mixins in `src/mixins/`, and assets.
- `hmod/src/drone/`: the game-agnostic drone simulation (physics, controller, collision, presets), unit-tested on its own.
- `hmod/src/setup/`: the setup screen; `model.rs` is its settings table and maths, unit-tested with the drone.
- `smod/`: the same mod as a Sunrise package: `sunrise-mod.toml` (the Sunrise pin and mixin targets) and `Cargo.toml` are hand-kept; `src/`, `assets/` and `LICENSE` are generated from `hmod/` by `python tools/sync_smod.py`, so edit `hmod/` and run it (`--check` fails if `smod/` is out of date).
- `tools/`: packaging and asset-generation scripts.

## Building

Run the core unit tests (no game files needed):

```bash
cargo test --lib
```

Package the mod into `dist/fpvhorizon.hmod`:

```bash
python tools/package.py
```

Package and install it into Horizon's native mods folder:

```bash
python tools/package.py --install
```

`python build.py` runs the tests, then packages and installs in one step.

## Releasing

Push a tag matching the version in `hmod/horizon-mod.toml` (for example `v0.9.0`). GitHub Actions builds the `.hmod` with `cargo horizon-mod build`, runs `cargo horizon-mod check` on it, and attaches it to a GitHub Release with its SHA-256. `0.x` versions are marked as pre-releases.

Without pushing a tag: Actions > Build > Run workflow on `master` with **release** ticked creates the `v<version>` tag on that commit and publishes the same release. It refuses a version that is already released.

### Sunrise

The Sunrise package is built by the **Build for Sunrise** workflow with Sunrise's own `cargo sunrise-mod build` against the game build `smod/sunrise-mod.toml` pins. Run it from Actions on `master` to just build (downloadable artifact), or tick **release** to also create the `smod-v<version>` tag and publish `fpvhorizon.smod` with its SHA-256. A tag `smod-v<version>` pushed by hand does the same. Locally, `cargo sunrise-mod check` in `smod/` type-checks it; `build` needs a MinGW toolchain on Windows.

## Credits

FPV Horizon's flight physics, drone build model, rates and controller logic (`hmod/src/drone/`) are based on code from [Minecraft FPV](https://github.com/minecraft-fpv/mfpv-client) by its authors, used under GPL-3.0.

## Licence

FPV Horizon is licensed under the GNU General Public License v3.0 only (GPL-3.0-only). See [LICENSE](LICENSE).
