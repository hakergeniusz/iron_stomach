// A click on the settings page: button 0 toggles guest protection, and the
// choice is saved to `config/iron_stomach/guests.txt` so the next process
// starts with it — the tick's sync-once reads the file back.
// Copyright (c) 2026 hakergeniusz. MIT.
fn iron_stomach_setting(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    button: usize,
) {
    use crate::hunger::iron_stomach_gate as gate;
    if mod_id != "iron_stomach" || button != 0 {
        return;
    }
    let on = !gate::protects_guests();
    gate::set_protects_guests(on);
    let _ = game;
    if let Ok(dir) = crate::game::Game::native_config_dir() {
        let dir = dir.join("iron_stomach");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("guests.txt"), if on { "on" } else { "off" });
    }
}
