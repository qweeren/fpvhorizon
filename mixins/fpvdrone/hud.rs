// native_hud hook: draws the FPV gimbal stick overlays on screen.
fn fpvdrone_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
) {
    if game.native_window() {
        crate::game::fpvdrone_adapter::hud(game, batch);
    }
}
