// native_key hook: handles keyboard input for drone toggle ('H') and WASD/arrow sticks.
fn fpvhorizon_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    if game.native_window() && game.native_playing() {
        crate::game::fpvhorizon_adapter::key(game, key, pressed);
    }
}
