// The local player's food tick, held whole: the drain, the hunger-driven
// healing inside it and the starvation damage it would push all stay as
// they were. Every other player's tick runs on untouched.
// Copyright (c) 2026 hakergeniusz. MIT.
fn iron_stomach_hold(player: &mut PlayerMotion) -> core::ops::ControlFlow<Result<()>> {
    if crate::hunger::iron_stomach_gate::holds(player.entity_id as u32) {
        return core::ops::ControlFlow::Break(Ok(()));
    }
    core::ops::ControlFlow::Continue(())
}
