// The local player's food tick, drained of its drain: the tick itself now
// RUNS, so the vanilla natural-regeneration blocks inside it keep healing —
// the game's own timers, amounts, gamerule and difficulty — while the food
// level stays frozen at full. The freeze is two writes before the tick body
// reads anything: the exhaustion the motion sim just charged (sprint, jump)
// is zeroed, so the tick's drain block (`exhaustion > 4.0`) never fires and
// nothing the player does ever reaches the food level, and the food level
// itself is floored at 20, so a player who was already starving is picked
// back up instead of starving forever. Starvation is therefore impossible,
// the regen cost (`add_exhaustion(6.0)` per heal) is dead by the next tick,
// and — while the window does not host a LAN world with guest protection —
// every other player's tick runs on untouched.
// Copyright (c) 2026 hakergeniusz. MIT.
fn iron_stomach_hold(player: &mut PlayerMotion) -> core::ops::ControlFlow<Result<()>> {
    if crate::hunger::iron_stomach_gate::freezing(player.entity_id as u32) {
        player.health.food.food = player.health.food.food.max(20);
        player.health.food.exhaustion = 0.0;
    }
    core::ops::ControlFlow::Continue(())
}
