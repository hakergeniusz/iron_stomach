//! Iron Stomach's frozen decision, as pure standard library: which entity id
//! is the local player, and whose food tick must therefore be held. The mod
//! itself is thin splices over Horizon's own food tick; this library carries
//! the rules and their tests.

/// The captured state's sentinel: no window game has published a local
/// player (no world open, or the decision cleared). An entity id of
/// `0x8000_0000` would encode to the same value, and ids never get there.
pub const NONE: i32 = i32::MIN;

/// Encode a captured local player id; `None` clears the decision.
pub fn encode(local: Option<u32>) -> i32 {
    match local {
        Some(id) => id as i32,
        None => NONE,
    }
}

/// Decode a captured state back into the local player id, if one is held.
pub fn decode(captured: i32) -> Option<u32> {
    (captured != NONE).then_some(captured as u32)
}

/// Whether the food tick of the entity `ticking` must be held whole: exactly
/// when it is the captured local player. Guests and mobs keep vanilla hunger,
/// and without a capture nobody is held.
pub fn freezes(captured: i32, ticking: u32) -> bool {
    decode(captured) == Some(ticking)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_and_captures_round_trip() {
        assert_eq!(encode(None), NONE);
        assert_eq!(decode(NONE), None);
        for id in [0u32, 1, 1298, 424_242, u32::MAX] {
            assert_eq!(decode(encode(Some(id))), Some(id));
        }
    }

    #[test]
    fn freezing_is_exact_for_the_captured_player_only() {
        let local = 424_242u32;
        let captured = encode(Some(local));
        assert!(freezes(captured, local));
        assert!(!freezes(captured, local + 1));
        assert!(!freezes(captured, local - 1));
        assert!(!freezes(captured, 0));
    }

    #[test]
    fn without_a_capture_nobody_freezes() {
        assert!(!freezes(NONE, 0));
        assert!(!freezes(NONE, 424_242));
    }
}
