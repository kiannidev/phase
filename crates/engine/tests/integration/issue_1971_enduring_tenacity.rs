//! Issue #1971 — Enduring Tenacity should return as an enchantment (not a creature)
//! when it dies as a creature.

use engine::game::scenario::{GameScenario, P0};
use engine::game::triggers::process_triggers;
use engine::types::card_type::CoreType;
use engine::types::game_state::WaitingFor;
use engine::types::zones::Zone;

const ENDURING_TENACITY_ORACLE: &str = "\
Whenever you gain life, target opponent loses that much life.\n\
When Enduring Tenacity dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)";

fn drain_to_priority(runner: &mut engine::game::scenario::GameRunner) {
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(
            guard < 256,
            "drain exceeded bound; waiting_for = {:?}",
            runner.state().waiting_for
        );
        match &runner.state().waiting_for {
            WaitingFor::Priority { .. } if runner.state().stack.is_empty() => break,
            _ => {
                if runner
                    .act(engine::types::actions::GameAction::PassPriority)
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

#[test]
fn enduring_tenacity_dies_returns_as_enchantment_only() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(engine::types::phase::Phase::PreCombatMain);

    let tenacity_id = scenario
        .add_creature_from_oracle(P0, "Enduring Tenacity", 3, 3, ENDURING_TENACITY_ORACLE)
        .id();

    let mut runner = scenario.build();

    let mut events = Vec::new();
    engine::game::zones::move_to_zone(runner.state_mut(), tenacity_id, Zone::Graveyard, &mut events);
    process_triggers(runner.state_mut(), &events);
    drain_to_priority(&mut runner);

    let returned = runner.state().objects.get(&tenacity_id).expect("object exists");
    assert_eq!(
        returned.zone,
        Zone::Battlefield,
        "Enduring Tenacity should return from graveyard; waiting_for = {:?}, stack = {}",
        runner.state().waiting_for,
        runner.state().stack.len()
    );
    assert!(
        returned.card_types.core_types.contains(&CoreType::Enchantment),
        "expected Enchantment, got {:?}",
        returned.card_types.core_types
    );
    assert!(
        !returned.card_types.core_types.contains(&CoreType::Creature),
        "should not remain a creature, got {:?}",
        returned.card_types.core_types
    );
}
