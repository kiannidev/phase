//! Regression for issue #3870: cancelling Yawgmoth's activated ability after
//! paying a sacrifice cost must return the sacrificed permanent.
//!
//! https://github.com/phase-rs/phase/issues/3870

use engine::game::scenario::{GameScenario, P0};
use engine::types::actions::GameAction;
use engine::types::game_state::{PayCostKind, WaitingFor};
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const YAWGMOTH_ORACLE: &str = "Protection from Humans\n\
Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target creature and draw a card.\n\
{B}{B}, Discard a card: Proliferate.";

#[test]
fn yawgmoth_cancel_after_sacrifice_cost_returns_token() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let yawgmoth = scenario
        .add_creature_from_oracle(P0, "Yawgmoth, Thran Physician", 2, 4, YAWGMOTH_ORACLE)
        .id();
    let spawn = scenario
        .add_creature(P0, "Eldrazi Spawn", 0, 1)
        .with_subtypes(vec!["Eldrazi", "Spawn"])
        .id();

    let mut runner = scenario.build();

    let ability_index = runner.state().objects[&yawgmoth]
        .abilities
        .iter()
        .position(|ability| {
            ability
                .description
                .as_deref()
                .is_some_and(|d| d.contains("Sacrifice another creature"))
        })
        .expect("Yawgmoth must expose the sacrifice ability");

    runner
        .act(GameAction::ActivateAbility {
            source_id: yawgmoth,
            ability_index,
        })
        .expect("begin Yawgmoth activation");

    let WaitingFor::PayCost {
        kind: PayCostKind::Sacrifice,
        choices,
        ..
    } = &runner.state().waiting_for
    else {
        panic!(
            "expected sacrifice PayCost after announcing Yawgmoth, got {:?}",
            runner.state().waiting_for
        );
    };
    assert!(
        choices.contains(&spawn),
        "Eldrazi Spawn must be legal sacrifice fodder"
    );

    runner
        .act(GameAction::SelectCards { cards: vec![spawn] })
        .expect("sacrifice Eldrazi Spawn for cost");

    assert_eq!(
        runner.state().objects[&spawn].zone,
        Zone::Graveyard,
        "precondition: sacrifice cost must move the spawn to the graveyard"
    );

    runner
        .act(GameAction::CancelCast)
        .expect("cancel after sacrifice cost");

    assert_eq!(
        runner.state().objects[&spawn].zone,
        Zone::Battlefield,
        "cancelled activation must return the sacrificed token to the battlefield"
    );
    assert!(
        matches!(runner.state().waiting_for, WaitingFor::Priority { .. }),
        "cancel must return to priority"
    );
}
