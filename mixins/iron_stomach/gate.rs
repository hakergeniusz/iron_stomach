// The captured decision the food tick reads: which entity id is this
// window's local player. The window game republishes it every tick (and
// clears it once its world is gone); the server thread's food ticks only
// read. Held state is one atomic, so the reader never blocks the writer.
// The same rules, with tests, live in the mod's `runtime` library; these
// staged tests prove them against Horizon's real food tick.
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Once;

/// No local player is published (`runtime::NONE`'s twin; this file cannot
/// reach that library — the bundle injects source, and the game's tree does
/// not depend on the mod crate).
pub const NONE: i32 = i32::MIN;

static CAPTURED: AtomicI32 = AtomicI32::new(NONE);

/// The window game publishes each tick; `None` clears the decision.
pub fn publish(local: Option<u32>) {
    CAPTURED.store(
        match local {
            Some(id) => id as i32,
            None => NONE,
        },
        Ordering::SeqCst,
    );
}

/// The captured decision: this window's local player id, if any.
pub fn captured() -> Option<u32> {
    let raw = CAPTURED.load(Ordering::SeqCst);
    (raw != NONE).then_some(raw as u32)
}

/// Whether the food tick of entity `ticking` must be held whole: exactly the
/// captured local player, nobody else — guests and mobs keep vanilla hunger.
pub fn holds(ticking: u32) -> bool {
    captured() == Some(ticking)
}

/// Whether the window's world is open to the network — the LAN host flag the
/// window's tick publishes and the server thread's food ticks read.
static HOSTING: AtomicBool = AtomicBool::new(false);
/// Whether guests on a hosted world fall under the mod too (the saved
/// choice; the default is on).
static PROTECT: AtomicBool = AtomicBool::new(true);

/// The window's tick publishes whether this world is open to the network.
pub fn hosting(on: bool) {
    HOSTING.store(on, Ordering::Release);
}

/// Whether the world is open to the network right now.
pub fn is_hosting() -> bool {
    HOSTING.load(Ordering::Acquire)
}

/// Whether guests on a hosted world fall under the mod.
pub fn protects_guests() -> bool {
    PROTECT.load(Ordering::Acquire)
}

/// The settings page's toggle (and the saved choice at startup) set this.
pub fn set_protects_guests(on: bool) {
    PROTECT.store(on, Ordering::Release);
}

/// Once per process: the saved choice (`<config>/iron_stomach/guests.txt`)
/// overrides the default. `None` just marks the sync done.
pub fn sync_saved(config_dir: Option<&Path>) {
    static DONE: Once = Once::new();
    DONE.call_once(|| {
        if let Some(dir) = config_dir {
            if let Ok(text) = fs::read_to_string(dir.join("iron_stomach").join("guests.txt")) {
                PROTECT.store(text.trim().eq_ignore_ascii_case("on"), Ordering::Release);
            }
        }
    });
}

/// Whether the food tick of entity `ticking` freezes: the captured local
/// player always, and — while the window hosts a LAN world with the guest
/// protection on — every player the server thread ticks. Mobs never pass
/// here: the hook sits on the players' food tick.
pub fn freezing(ticking: u32) -> bool {
    holds(ticking) || (PROTECT.load(Ordering::Acquire) && HOSTING.load(Ordering::Acquire))
}

#[cfg(test)]
mod tests {
    use super::{captured, holds, publish};
    use crate::assets::Assets;
    use crate::health::{HealthEvent, HealthRules};
    use crate::motion::{MotionEvent, MotionResources, PlayerMotion};
    use horizon_core::geometry::Vec3;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    /// Entity ids the game would never hand out, so the staged assertions
    /// cannot collide with real ids.
    const LOCAL: u32 = 424_242;
    const GUEST: u32 = 424_243;

    fn resources() -> &'static MotionResources {
        static RESOURCES: OnceLock<MotionResources> = OnceLock::new();
        RESOURCES.get_or_init(|| {
            let root = PathBuf::from(
                std::env::var_os("HORIZON_TEST_RESOURCES")
                    .expect("the tooling sets HORIZON_TEST_RESOURCES for the staged suite"),
            );
            let assets = Assets::load(&root).expect("prepared assets load");
            MotionResources::load(&root, &assets).expect("prepared motion resources load")
        })
    }

    /// Sets the decision for the test's body and always clears it after, so
    /// no other test in this binary inherits a capture.
    struct Decision(());
    impl Decision {
        fn on() -> Self {
            publish(Some(LOCAL));
            Self(())
        }
        fn off() -> Self {
            publish(None);
            Self(())
        }
    }
    impl Drop for Decision {
        fn drop(&mut self) {
            publish(None);
        }
    }

    fn player(id: u32, food: i32, exhaustion: f32, timer: i32) -> PlayerMotion {
        let mut player = PlayerMotion::new(resources(), Vec3::ZERO).expect("player builds");
        player.entity_id = id as i32;
        player.health.food.food = food;
        player.health.food.saturation = 0.0;
        player.health.food.exhaustion = exhaustion;
        player.health.food.timer = timer;
        player
    }

    #[test]
    fn publish_and_holds_track_the_captured_id() {
        let _decision = Decision::off();
        assert_eq!(captured(), None);
        assert!(!holds(LOCAL));
        publish(Some(LOCAL));
        assert_eq!(captured(), Some(LOCAL));
        assert!(holds(LOCAL));
        assert!(!holds(GUEST));
        publish(None);
        assert_eq!(captured(), None);
    }

    /// The frozen player's tick now RUNS: the hook zeroes the exhaustion
    /// before the body reads it, so the drain block never fires and the
    /// vanilla regeneration blocks inside the tick keep working. At full
    /// health nothing heals, and the food level never moves.
    #[test]
    fn the_frozen_player_loses_nothing_and_the_tick_runs() {
        let _decision = Decision::on();
        let mut local = player(LOCAL, 20, 5.0, 79);
        local
            .food_tick(&resources().health, HealthRules::default())
            .unwrap();
        assert_eq!(local.health.food.food, 20, "the food level never drops");
        assert_eq!(local.health.food.exhaustion, 0.0, "the drain never fires");
        assert_eq!(local.health.food.timer, 0, "no branch held the timer");
        assert_eq!(local.health.health(), 20.0);
        assert!(local.events.is_empty(), "no starvation, no healing");
    }

    /// The point of 1.1.0: the frozen player still regenerates — the game's
    /// own slow-regen block (food ≥ 18, one heart per four seconds, the
    /// gamerule and difficulty as the server holds them) heals while the
    /// food level stays frozen, and the heal's own exhaustion cost is dead
    /// by the next tick.
    #[test]
    fn the_frozen_player_still_regenerates_naturally() {
        let _decision = Decision::on();
        let mut local = player(LOCAL, 20, 0.0, 0);
        local.set_health(10.0, false).expect("hurt to 10");
        for _ in 0..81 {
            local
                .food_tick(&resources().health, HealthRules::default())
                .unwrap();
        }
        assert!(
            local.health.health() >= 11.0,
            "the tick's own regen healed: {}",
            local.health.health()
        );
        assert_eq!(local.health.food.food, 20, "the food level stays frozen through regen");
        assert_eq!(
            local.health.food.exhaustion, 0.0,
            "the regen's cost is dead by the tick that follows"
        );
    }

    #[test]
    fn guest_drains_in_the_same_state() {
        let _decision = Decision::on();
        let mut guest = player(GUEST, 20, 5.0, 79);
        guest
            .food_tick(&resources().health, HealthRules::default())
            .unwrap();
        assert_eq!(guest.health.food.food, 19, "vanilla hunger for guests");
        assert_eq!(guest.health.food.exhaustion, 1.0);
    }

    #[test]
    fn without_a_capture_the_local_drains_too() {
        let _decision = Decision::off();
        let mut local = player(LOCAL, 20, 5.0, 79);
        local
            .food_tick(&resources().health, HealthRules::default())
            .unwrap();
        assert_eq!(local.health.food.food, 19);
        assert_eq!(local.health.food.exhaustion, 1.0);
    }

    #[test]
    fn starvation_is_held_for_the_local_and_bites_the_guest() {
        let _decision = Decision::on();
        let mut local = player(LOCAL, 0, 0.0, 79);
        local
            .food_tick(&resources().health, HealthRules::default())
            .unwrap();
        assert_eq!(
            local.health.food.food, 20,
            "frozen back to full before the tick"
        );
        assert_eq!(local.health.health(), 20.0, "no starvation damage");
        assert!(local.events.is_empty());
        assert_eq!(local.health.food.timer, 0);
        drop(_decision);

        let mut guest = player(GUEST, 0, 0.0, 79);
        guest
            .food_tick(&resources().health, HealthRules::default())
            .unwrap();
        assert!(
            guest.events.iter().any(|event| matches!(
                event,
                MotionEvent::Health(HealthEvent::StarvationDamage(_))
            )),
            "the guest starves"
        );
        assert_eq!(guest.health.health(), 19.0);
        assert_eq!(captured(), None, "the dropped guard cleared the decision");
    }
}
