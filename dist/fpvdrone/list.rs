// native_mods hook: registers the mod in Options > Mods.
fn fpvdrone_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::fpvdrone_adapter::list(mods);
}
