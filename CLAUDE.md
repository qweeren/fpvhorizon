# FPV Horizon

A native Horizon mod for flying FPV drones. Owner: qweeren (Eren). Default branch: `master`.

## Standing permissions

These are durable approvals from the owner. Act on them without asking first.

- **Commit, push and merge to `master`** without asking. Work on the session branch if one is required, then
  fast-forward or merge it into `master` and push. Never force-push `master` or rewrite its history.
- **Release builds**: bump the version in `hmod/horizon-mod.toml` (and both `Cargo.toml`s) when asked for a release,
  then run the **Build** workflow on `master` with `release` ticked. Don't release a version the owner hasn't asked
  for or hasn't tested.
- **Fix failing checks and review comments on this repo's own pull requests** and push the fixes. Never skip, disable
  or weaken a test to get green.
- **Keep `ROADMAP.md` up to date** as work lands.
- **The registry fork** (`qweeren/horizon-mods`, branch per submission): commit and push entry updates there without
  asking, since they only update the owner's own pull request.

Still ask first for anything public under the owner's name on **other people's repositories**: opening pull requests
or issues, posting comments or reviews. Draft the text and let the owner post or approve it.

## Layout

- `hmod/`: the whole mod package. `horizon-mod.toml` (manifest, pins), the crate in `src/`, mixin helpers in
  `src/mixins/`, assets in `assets/fpvhorizon/`.
- `hmod/src/drone/`: the game-agnostic simulation (physics, controller, collision, presets). Unit-tested on its own
  through `tools/standalone.rs`.
- `tools/`: `package.py` (quick local package/install), asset generators.
- `art/icon.png`: the registry icon (temporary).

## Build, test, release

- Tests: `cargo test --lib` from the repo root. Run them before pushing code changes.
- Official package: `cargo horizon-mod build` in `hmod/`, then `cargo horizon-mod check target/horizon-mod/fpvhorizon.hmod`.
  CI does this in `.github/workflows/release.yml` with `cargo-horizon-mod` from the Horizon commit pinned in
  `[native].horizon`.
- Local in-game builds: `horizon-launcher native-build --instance <id> --version <tag>`. It takes ~5 min cached,
  never more than 15-20. Always run it with a 20-minute cap and a stall check (log quiet ~5 min with no
  rustc/cargo = hung, kill it). When it stalls at "Compiling fpvhorizon (1 of 1)", read
  `%APPDATA%\horizon-launcher
ativeuild.log`: the launcher hides compile errors there and hangs instead of
  failing (seen with E0603 private imports on launcher 0.2.81; fixed in launcher 0.2.82, horizon#18509).
- Cloud sessions can't push tags (proxy 403). Release through the workflow's `release` input instead.
- The Horizon pin must match `horizon-rust/horizon-mods`'s `.github/horizon-rev` for the registry to accept a build.

## Registry

Published through pull requests to `horizon-rust/horizon-mods` (the web form doesn't take `.hmod` files yet).
Entry: `mods/fpvhorizon.toml` on the fork. Each new version adds a `[[versions]]` record with the release file's
SHA-256, and every native version is reviewed again.

## Conventions

- Mod id and asset namespace: `fpvhorizon`. Display name: FPV Horizon.
- GPL-3.0-only. FPV Horizon is its own mod, not a port. Its physics code is based on
  [Minecraft FPV](https://github.com/minecraft-fpv/mfpv-client) (GPL-3.0): keep that credit at the bottom of the README
  and in the drone files' headers; the GPL requires it. Don't call the project a port.
- Mobile (Starling) is planned for when Starling reads native `.hmod`s; see `ROADMAP.md`.
