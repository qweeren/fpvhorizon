// native_settings_click hook: handles button clicks on the settings page.
fn fpvdrone_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id == "fpvdrone" && game.native_window() {
        crate::game::fpvdrone_adapter::click(game, button);
    }
}
