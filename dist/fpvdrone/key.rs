// native_key hook: handles keyboard input for drone toggle ('H') and WASD/arrow sticks.
fn fpvdrone_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    if game.native_window() && game.native_playing() {
        crate::game::fpvdrone_adapter::key(game, key, pressed);
    }
}
