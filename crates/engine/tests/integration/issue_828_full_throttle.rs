//! Issue #828 — Full Throttle must schedule extra combat phases after the main phase.
//!
//! https://github.com/phase-rs/phase/issues/828

use engine::game::scenario::{GameScenario, P0};
use engine::types::mana::ManaCost;
use engine::types::phase::Phase;

const FULL_THROTTLE: &str = "After this main phase, there are two additional combat phases.
At the beginning of each combat this turn, untap all creatures that attacked this turn.";

#[test]
fn full_throttle_schedules_two_extra_combats_after_main_phase() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let throttle = scenario
        .add_spell_to_hand_from_oracle(P0, "Full Throttle", false, FULL_THROTTLE)
        .with_mana_cost(ManaCost::generic(0))
        .id();

    let mut runner = scenario.build();
    runner.cast(throttle).resolve();
    runner.advance_until_stack_empty();

    assert_eq!(
        runner.state().extra_phases.len(),
        2,
        "Full Throttle must schedule two extra combat phases after the current main phase"
    );
    assert!(
        runner
            .state()
            .extra_phases
            .iter()
            .all(|ep| ep.anchor == Phase::PreCombatMain && ep.phase == Phase::BeginCombat),
        "extra combats must anchor to the main phase that Full Throttle resolved in"
    );
}

#[test]
fn full_throttle_postcombat_main_anchors_to_postcombat_main() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PostCombatMain);
    let throttle = scenario
        .add_spell_to_hand_from_oracle(P0, "Full Throttle", false, FULL_THROTTLE)
        .with_mana_cost(ManaCost::generic(0))
        .id();

    let mut runner = scenario.build();
    runner.cast(throttle).resolve();
    runner.advance_until_stack_empty();

    assert_eq!(runner.state().extra_phases.len(), 2);
    assert!(
        runner
            .state()
            .extra_phases
            .iter()
            .all(|ep| ep.anchor == Phase::PostCombatMain && ep.phase == Phase::BeginCombat),
        "casting in postcombat main must anchor extra combats to postcombat main, not end of combat"
    );
}
