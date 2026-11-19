// native_settings hook: lists settings buttons on the mod's configuration page.
fn fpvhorizon_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == "fpvhorizon" && game.native_window() {
        crate::game::fpvhorizon_adapter::settings(game, buttons);
    }
}
