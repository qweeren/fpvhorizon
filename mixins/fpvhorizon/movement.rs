// native_movement_input hook: zeroes normal walking/jumping inputs when drone flight is active.
fn fpvhorizon_movement(
    game: &crate::game::Game<'_>,
    forward: &mut f32,
    strafe: &mut f32,
    jump: &mut bool,
    sneak: &mut bool,
) {
    crate::game::fpvhorizon_adapter::movement(game, forward, strafe, jump, sneak);
}
