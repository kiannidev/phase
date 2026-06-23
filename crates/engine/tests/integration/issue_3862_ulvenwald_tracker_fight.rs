//! Regression for issue #3862: Ulvenwald Tracker must make two chosen creatures
//! fight each other, not fight the Tracker itself.
//!
//! https://github.com/phase-rs/phase/issues/3862

use engine::game::scenario::{GameScenario, P0, P1};
use engine::types::identifiers::ObjectId;
use engine::types::mana::{ManaType, ManaUnit};
use engine::types::phase::Phase;

const ULVENWALD_TRACKER_ORACLE: &str =
    "{1}{G}, {T}: Target creature you control fights another target creature.";

fn floating_mana(n: usize, ty: ManaType) -> Vec<ManaUnit> {
    (0..n)
        .map(|_| ManaUnit::new(ty, ObjectId(0), false, vec![]))
        .collect()
}

#[test]
fn ulvenwald_tracker_dual_target_fight_does_not_include_tracker() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);

    let tracker = scenario
        .add_creature_from_oracle(P0, "Ulvenwald Tracker", 1, 1, ULVENWALD_TRACKER_ORACLE)
        .id();
    let bear = scenario.add_creature(P0, "Bear", 3, 3).id();
    let wolf = scenario.add_creature(P1, "Wolf", 2, 2).id();
    scenario.with_mana_pool(
        P0,
        floating_mana(1, ManaType::Colorless)
            .into_iter()
            .chain(floating_mana(1, ManaType::Green))
            .collect(),
    );

    let mut runner = scenario.build();

    runner
        .activate(tracker, 0)
        .target_objects(&[bear, wolf])
        .resolve();

    assert_eq!(
        runner.state().objects[&wolf].damage_marked,
        3,
        "Bear (3 power) must deal 3 damage to Wolf"
    );
    assert_eq!(
        runner.state().objects[&bear].damage_marked,
        2,
        "Wolf (2 power) must deal 2 damage to Bear"
    );
    assert_eq!(
        runner.state().objects[&tracker].damage_marked,
        0,
        "Ulvenwald Tracker is not a fighter in this fight"
    );
}
