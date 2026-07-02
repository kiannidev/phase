//! Issue #218: Dogmeat, Ever Loyal must mill five cards on ETB.

use engine::game::scenario::{GameScenario, P0};
use engine::types::phase::Phase;

const DOGMEAT_ORACLE: &str = "\
When Dogmeat enters, mill five cards, then return an Aura or Equipment card from your graveyard to your hand.\n\
Whenever a creature you control that's enchanted or equipped attacks, create a Junk token. (It's an artifact with \"{T}, Sacrifice this token: Exile the top card of your library. You may play that card this turn. Activate only as a sorcery.\")";

#[test]
fn dogmeat_etb_mills_five_cards() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    for i in 0..10 {
        scenario.with_library_top(P0, &[&format!("Library Card {i}")]);
    }

    let dogmeat = scenario
        .add_creature_to_hand_from_oracle(P0, "Dogmeat, Ever Loyal", 3, 3, DOGMEAT_ORACLE)
        .id();

    let mut runner = scenario.build();
    let library_before = runner.state().players[0].library.len();
    let graveyard_before = runner.state().players[0].graveyard.len();

    runner.cast(dogmeat).resolve();
    runner.advance_until_stack_empty();

    assert_eq!(
        runner.state().players[0].library.len(),
        library_before - 5,
        "Dogmeat ETB must mill exactly five cards"
    );
    assert_eq!(
        runner.state().players[0].graveyard.len(),
        graveyard_before + 5,
        "milled cards must land in graveyard"
    );
}
