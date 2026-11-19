fn fpvdrone_tick(game: &mut Game<'_>) {
    crate::game::native_hooks::client_tick_post::fpvdrone_adapter::tick(game);
}
