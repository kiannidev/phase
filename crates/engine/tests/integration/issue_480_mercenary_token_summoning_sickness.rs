//! Issue #480: Mercenary tokens created this turn must have summoning sickness.

use engine::game::scenario::{GameScenario, P0};
use engine::types::actions::GameAction;
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const PRICKLY_PAIR_ORACLE: &str = "\
When this creature enters, create a 1/1 red Mercenary creature token with \"{T}: Target creature you control gets +1/+0 until end of turn. Activate only as a sorcery.\"";

#[test]
fn mercenary_token_from_prickly_pair_has_summoning_sickness() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let host = scenario.add_vanilla(P0, 2, 2);
    let prickly = scenario
        .add_creature_to_hand_from_oracle(P0, "Prickly Pair", 2, 2, PRICKLY_PAIR_ORACLE)
        .id();

    let mut runner = scenario.build();
    runner.cast(prickly).resolve();
    runner.advance_until_stack_empty();

    let mercenary = runner
        .state()
        .objects
        .values()
        .find(|o| {
            o.is_token
                && o.zone == Zone::Battlefield
                && o.card_types.subtypes.iter().any(|s| s == "Mercenary")
        })
        .expect("Prickly Pair ETB must create a Mercenary token");

    assert!(
        mercenary.summoning_sick,
        "Mercenary token must enter with summoning sickness"
    );

    let tap_index = 0;

    let err = runner
        .act(GameAction::ActivateAbility {
            source_id: mercenary.id,
            ability_index: tap_index,
        })
        .expect_err("summoning-sick Mercenary token must not tap for its ability");
    assert!(
        matches!(err, engine::game::EngineError::ActionNotAllowed(_)),
        "expected ActionNotAllowed, got {err:?}"
    );

    // Sanity: host creature without sickness can still be targeted later.
    assert_eq!(runner.state().objects[&host].zone, Zone::Battlefield);
}
