# Iron Stomach

An always-on Horizon native mod that holds the local player's hunger: **the
food level never drops** and **starvation never hurts** — sprinting, jumping,
swimming, mining, combat and status effects included, and a bar already at
zero stays harmless. Everyone else keeps vanilla hunger: LAN guests still
drain their food and can starve, and so do mobs. No settings and no UI —
there is no Options > Mods row; the mod is always active.

## Scope (the honest limits)

- **Client-side, this process only.** The mod controls worlds simulated
  inside its own process: singleplayer worlds and worlds you host over LAN —
  those are the worlds whose server-side food tick it holds.
- **Dedicated servers are untouched.** There the server process owns the
  food data; a client mod cannot change it, and Iron Stomach does not try.
  Eating, saturation and food properties are all vanilla.
- **Accepted trade-off:** health regeneration that is driven by the food
  tick stops working while the food level is held (the held tick is skipped
  whole). Peaceful-difficulty regeneration is a separate code path and keeps
  working. The exhaustion counter keeps accruing while held; vanilla caps it
  at 40.

## How it fits together

Horizon runs hunger server-side: `PlayerMotion::food_tick` drains food,
drives hunger-cost regeneration and pushes starvation damage, and the
window's own game never runs it (it returns early for client-side players).
Iron Stomach splices two thin helpers into that path:

- every tick, the window game publishes its local player's entity id (from
  `game.remote`) into one atomic, and clears it once no world is open;
- a cancellable head on `food_tick` holds exactly that player's tick whole
  and lets every other player's tick run untouched.

The decision rules are pure standard library in
`mixins/iron_stomach/runtime.rs`, doubled as the crate's library and tested
standalone (3 tests); the injected gate module's staged tests drive a real
`food_tick` over prepared resources and prove the local player's food stays
put while a guest's drains in the very same state (5 tests).

Verification recipe:

```sh
python3 -I tools/test.py           # fmt, clippy, the decision rules' tests
python3 -I tools/horizon.py check  # staged cargo check on the pinned revision
python3 -I tools/horizon.py test   # staged behavior tests over a real food_tick
python3 -I tools/package.py        # target/iron_stomach-1.0.0.hnmod
```

## Provenance

Clean-room: the behavior follows the no-hunger utility genre — the idea
only. No other mod's code, text or assets was consulted or copied;
Iron Stomach is implemented from a behavior spec against Horizon's own mod
API (native mixin hooks on the pinned revision).

## License

MIT — see [LICENSE](LICENSE).
