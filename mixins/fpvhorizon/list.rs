// native_mods hook: registers the mod in Options > Mods.
fn fpvhorizon_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::fpvhorizon_adapter::list(mods);
}
