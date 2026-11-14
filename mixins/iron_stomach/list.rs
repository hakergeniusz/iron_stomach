// The mod's row in Options > Mods: the mod's identity and what it does. No
// keybinds — the one toggle lives on the settings page.
// Copyright (c) 2026 hakergeniusz. MIT.
fn iron_stomach_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "iron_stomach".into(),
        title: "Iron Stomach 1.2.0".into(),
        description: "No hunger drain for you — and for every player on a world you open to LAN (toggle on the settings page). Natural regeneration still works.".into(),
        keys: vec![],
    });
}
