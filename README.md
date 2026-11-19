# FPV Horizon

FPV Horizon is a native Horizon mod for flying FPV drones. It brings realistic quadcopter physics, Betaflight-style rates, and acro flight mechanics inside Horizon.

## Features

- **Quadcopter physics**:
  - Full 6-DoF aerodynamic simulation.
  - Blade element theory with multi-section propeller lift and drag curves.
  - Electric motor dynamics with back-EMF, winding resistance, and thermal modelling (Joule heating and convective cooling).
  - Battery voltage sag under throttle.
  - Block collisions with glancing angles and elastic bouncing.
- **Real drone presets**: 5" freestyle and race quads, a 3.5" cinewhoop, a 75mm whoop and a 7.5" long-range build, or your own custom build.
- **Betaflight rates**: RC Rate, Super Rate and Expo for roll, pitch and yaw, configurable deadzone, and an optional 3D mode (reversible throttle).
- **FPV view**: the camera rolls with the drone, with an adjustable field of view, an OSD and stick gimbal overlays.
- **Gamepad support**: fly with a controller's sticks, or with mouse and keyboard.
- **Motor and wind sound**.

## Controls

| Keyboard | Gamepad | Action |
|---|---|---|
| **H** | **L1 + R1** | Put the drone down and switch to its camera / return to your own eyes |
| **R** | **Triangle** | Arm / disarm |
| **V** | **D-pad up** | Switch view |
| **[** / **]** | **D-pad left / right** | Change drone (on the ground) |
| **Backspace** | **Square** | Reset the drone |
| **Mouse X / Y** | Right stick | Roll and pitch |
| **W / S** (or **Up / Down**) | Left stick up / down | Throttle |
| **A / D** (or **Left / Right**) | Left stick left / right | Yaw |

## Settings (Options > Mods > FPV Horizon)

- **Drone**: pick a preset, or turn on **Custom Build** to set mass, battery, motors and props with the Build sliders.
- **Rates**: RC Rate, Super Rate and Expo per axis.
- **Flight**: mouse sensitivity, deadzone, gravity, 3D mode, and stick-centre hover for gamepads.
- **View**: FPV field of view, video range, hiding the HUD while flying, and the stick gimbals.
- **Sound**: motor and wind volume.
- **Physics**: air density, drag and motor thermal constants.

## Project layout

- `core/`: the game-agnostic drone simulation (physics, controller, collision, presets), unit-tested on its own.
- `hmod/`: the Horizon mod: `horizon-mod.toml`, the mod crate in `src/`, its mixins in `src/mixins/`, and assets.
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

Push a tag matching the version in `hmod/horizon-mod.toml` (for example `v2.0.0`). GitHub Actions builds the `.hmod` and attaches it to a GitHub Release.

## Credits

The flight physics, drone builds, rates and controller logic in `core/` started as a Rust port of the [Minecraft FPV](https://github.com/minecraft-fpv/mfpv-client) mod by its authors, licensed under GPL-3.0. FPV Horizon has since been rebuilt for Horizon with its own models, sounds, OSD, presets and gamepad support.

## Licence

FPV Horizon is licensed under the GNU General Public License v3.0 only (GPL-3.0-only). See [LICENSE](LICENSE).
