// native_tick hook: advances physics, processes controls and orients camera.
fn fpvdrone_tick(game: &mut crate::game::Game<'_>) {
    if game.native_window() {
        crate::game::fpvdrone_adapter::tick(game);
    }
}
