// Each tick, the window game publishes who its local player is (and clears
// the decision once no world is open); server games — the single player
// server thread, dedicated servers — leave the decision alone. The food
// tick's hold reads it from the server thread.
// Copyright (c) 2026 hakergeniusz. MIT.
fn iron_stomach_capture(game: &mut crate::game::Game<'_>) {
    if game.native_window() {
        crate::hunger::iron_stomach_gate::publish(
            game.remote.as_ref().map(|remote| remote.entity_id as u32),
        );
    }
}
