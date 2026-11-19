// native_tick hook: advances physics, processes controls and orients camera.
fn fpvhorizon_tick(game: &mut crate::game::Game<'_>) {
    if game.native_window() {
        crate::game::fpvhorizon_adapter::tick(game);
    }
}
