// native_settings hook: lists settings buttons on the mod's configuration page.
fn fpvdrone_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == "fpvdrone" && game.native_window() {
        crate::game::fpvdrone_adapter::settings(game, buttons);
    }
}
