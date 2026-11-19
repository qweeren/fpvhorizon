fn fpvhorizon_tick(game: &mut Game<'_>) {
    crate::game::native_hooks::client_tick_post::fpvhorizon_adapter::tick(game);
}
