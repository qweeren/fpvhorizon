# Roadmap

## 0.9.0 beta (in progress)

- [x] Rename to FPV Horizon, remove the old `.hnmod` format
- [x] GPL-3.0 licence, credit to Minecraft FPV
- [x] Release workflow: `cargo horizon-mod build` + `check`, SHA-256 in the release
- [x] Temporary icon
- [ ] Trial build passes in GitHub Actions, tag `v0.9.0`
- [ ] Submit to the horizon-mods registry (`mods/fpvhorizon.toml` on the fork)

## Polish before 1.0

- [ ] Final icon and screenshots for the mod page
- [ ] Use Horizon's native gamepad button icons in the OSD hints instead of text ("D-pad left", "Triangle",
      "L1+R1" in `hmod/src/osd.rs`), and show only the keyboard or the gamepad hint depending on which is in use
- [ ] Sign releases with minisign

## Mobile: Starling (Android)

[Starling](https://github.com/starling-client/starling) is the Android fork of Horizon. It loads native mods only as the
old `.hnmod` format today. Native `.hmod` support is expected, since the Horizon and Starling maintainers are
coordinating on the new mod structure, so the plan is to **wait for it** rather than keep a second `.hnmod` build.

When Starling reads native `.hmod`s:

- [ ] Check that the mod compiles through Starling's build service (ARM64) and runs on a phone
- [ ] Gamepad flying with a Bluetooth controller (Starling already supports Android gamepads)
- [ ] **Two analog touch sticks** while flying, Mode 2: throttle and yaw on the left, pitch and roll on the right,
      with a throttle stick that stays where it's left (no spring to centre)
- [ ] On-screen buttons for arm, reset, switch view and change drone; hide Starling's own touch controls while flying
- [ ] Touch settings: stick size, position, opacity and deadzone
- [ ] Only show the touch layer on touch devices, so desktop Horizon is unaffected

Groundwork that doesn't need Starling:

- [ ] Design the touch stick layout and how it feeds the existing `Sticks` input
- [ ] Find which Starling touch code (`game/touch/`, `mobile_controls.rs`) the mod must hook, and watch it for changes
