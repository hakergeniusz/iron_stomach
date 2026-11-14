// Each tick, the window game publishes who its local player is (and clears
// the decision once no world is open); server games — the single player
// server thread, dedicated servers — leave the decision alone. The window
// also publishes whether its world is open to the network (the LAN host
// flag the food tick's freeze reads) and, once per process, syncs the saved
// guest-protection choice. The food tick's hold reads all of it from the
// server thread.
// Copyright (c) 2026 hakergeniusz. MIT.
fn iron_stomach_capture(game: &mut crate::game::Game<'_>) {
    if game.native_window() {
        match crate::game::Game::native_config_dir() {
            Ok(dir) => crate::hunger::iron_stomach_gate::sync_saved(Some(&dir)),
            Err(_) => crate::hunger::iron_stomach_gate::sync_saved(None),
        }
        crate::hunger::iron_stomach_gate::hosting(game.lan.is_some());
        crate::hunger::iron_stomach_gate::publish(
            game.remote.as_ref().map(|remote| remote.entity_id as u32),
        );
    }
}
