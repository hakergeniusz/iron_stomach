//! Iron Stomach: hunger never drains. The `food_level_changed` hook cancels
//! every change to a server player's food level, so eating, hunger and
//! saturation effects, exhaustion and starving never move it - for the local
//! player always, and for every guest on a hosted LAN world while "Protect
//! guests" is on.

use mod_api::{Flow, Hooks, RegistrationContext, Registrar, ServerHost, SettingSpec};

const ID: &str = "iron_stomach";

#[derive(Default)]
pub struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// The Options > Mods page's "Protect guests" toggle (default on).
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        let _ = registrar.register_setting(SettingSpec::toggle(
            "guests",
            format!("{ID}.setting.guests"),
            true,
        ));
    }

    /// The food level of a server player is about to change: freeze it.
    /// The acting player is the world's local player (single-player, or the
    /// LAN host's own view of themselves); every other server player is a
    /// guest on this host's world.
    fn food_level_changed(
        &self,
        host: &mut dyn ServerHost,
        event: &mut mod_api::FoodLevelChangedEvent<'_>,
    ) -> Flow {
        let local = host.players().acting_player().and_then(|p| p.name(host));
        let is_local = local.as_deref() == Some(event.name);
        if is_local || host.setting(ID, "guests").as_deref() == Some("true") {
            event.cancelled = true;
        }
        Flow::Continue
    }
}
