// native_settings_click hook: handles button clicks on the settings page.
fn fpvhorizon_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id == "fpvhorizon" && game.native_window() {
        crate::game::fpvhorizon_adapter::click(game, button);
    }
}
