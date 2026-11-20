# Roadmap

## 0.9.0 beta (in progress)

- [x] Rename to FPV Horizon, remove the old `.hnmod` format
- [x] GPL-3.0 licence, credit to Minecraft FPV
- [x] Release workflow: `cargo horizon-mod build` + `check`, SHA-256 in the release
- [x] Temporary icon
- [x] Trial build passes in GitHub Actions, release `v0.9.0`
- [x] Submit to the horizon-mods registry: [horizon-mods#1319](https://github.com/horizon-rust/horizon-mods/pull/1319)
- [ ] Registry review and merge

## 1.0.0

- [x] Release `v1.0.0` (Nightly 2026-11-20, 0a875da)
- [x] Release `v1.0.1`: OSD hints follow Nintendo letter mode (from review)
- [x] Release `v1.1.0`: the setup screen (not added to the registry entry)
- [ ] Registry review and merge: [horizon-mods#1467](https://github.com/horizon-rust/horizon-mods/pull/1467)

## Polish after 1.0

- [ ] Final icon and screenshots for the mod page
- [x] Use Horizon's native gamepad button icons in the OSD hints instead of text ("D-pad left", "Triangle",
      "L1+R1" in `hmod/src/osd.rs`), and show only the keyboard or the gamepad hint depending on which is in use
- [x] Fisheye lens on the FPV camera (Off / Light / Medium / Strong in the settings)
- [x] Setup screen (J, a pad's Select, or the pause menu): every setting with live previews, in the OSD's style
- [x] Flight modes: Acro, Angle, Horizon and 3D, switched with L1 (or M) as in Liftoff; shown on the OSD
- [x] Unlimited video range (the slider's top stop), and battery and motor heat simulation toggles
- [ ] Load the world around the drone in single player: waits on a Horizon mod API to move where a player's
      chunks stream from ([horizon#19219](https://github.com/horizon-rust/horizon/issues/19219)); neither chunk
      tickets, chunk areas nor mixins can do it on the current build
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
