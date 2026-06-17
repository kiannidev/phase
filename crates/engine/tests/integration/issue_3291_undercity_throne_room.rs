//! Regression for GitHub issue #3291 — Undercity room 8 (Throne of the Dead Three).
//!
//! Oracle: "Reveal the top ten cards of your library. Put a creature card from
//! among them onto the battlefield with three +1/+1 counters on it. It gains
//! hexproof until your next turn. Then shuffle."
//!
//! Bug: room effect was still `Effect::Unimplemented`, so venturing into the
//! final room did nothing.

use engine::game::dungeon::{room_effects, DungeonId};
use engine::game::scenario::P0;
use engine::types::ability::Effect;
use engine::types::identifiers::ObjectId;

#[test]
fn undercity_throne_room_is_implemented() {
    let (ability, _) = room_effects(DungeonId::Undercity, 8, ObjectId(1), P0);
    assert!(
        !matches!(ability.effect, Effect::Unimplemented { .. }),
        "Throne of the Dead Three must not be Unimplemented, got {:?}",
        ability.effect
    );
    assert!(
        matches!(ability.effect, Effect::Dig { reveal: true, .. }),
        "Throne must reveal and dig from library top ten, got {:?}",
        ability.effect
    );
}
