//! Regression (issue #581): Mystic Remora must add an age counter at upkeep and
//! prompt its controller to pay cumulative upkeep {1} per age counter.

use engine::game::scenario::GameScenario;
use engine::types::ability::AbilityCost;
use engine::types::actions::GameAction;
use engine::types::counter::CounterType;
use engine::types::game_state::WaitingFor;
use engine::types::identifiers::ObjectId;
use engine::types::mana::{ManaCost, ManaType, ManaUnit};
use engine::types::phase::Phase;
use engine::types::player::PlayerId;
use engine::types::triggers::TriggerMode;
use engine::types::zones::Zone;

const P0: PlayerId = PlayerId(0);

const MYSTIC_REMORA_ORACLE: &str = "Cumulative upkeep {1} (At the beginning of your upkeep, put an age counter on this permanent, then sacrifice it unless you pay its upkeep cost for each age counter on it.)\nWhenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}.";

fn floating_colorless(n: usize) -> Vec<ManaUnit> {
    (0..n)
        .map(|_| ManaUnit::new(ManaType::Colorless, ObjectId(0), false, vec![]))
        .collect()
}

fn setup_at_unless_prompt() -> (engine::game::scenario::GameRunner, ObjectId) {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::Untap);

    let remora = scenario
        .add_creature_from_oracle(P0, "Mystic Remora", 0, 0, MYSTIC_REMORA_ORACLE)
        .as_artifact()
        .id();

    let mut runner = scenario.build();

    assert!(
        runner
            .state()
            .objects
            .get(&remora)
            .unwrap()
            .trigger_definitions
            .as_slice()
            .iter()
            .any(|t| matches!(t.mode, TriggerMode::PayCumulativeUpkeep)),
        "Mystic Remora must carry a synthesized cumulative-upkeep trigger"
    );

    runner.advance_to_upkeep();
    runner.resolve_top();
    (runner, remora)
}

#[test]
fn issue_581_mystic_remora_upkeep_adds_age_counter_and_prompts_payment() {
    let (mut runner, remora) = setup_at_unless_prompt();

    assert_eq!(
        runner.state().objects[&remora]
            .counters
            .get(&CounterType::Age)
            .copied(),
        Some(1),
        "first upkeep must add one age counter before the unless prompt"
    );

    match &runner.state().waiting_for {
        WaitingFor::UnlessPayment { player, cost, .. } => {
            assert_eq!(*player, P0);
            assert_eq!(
                cost,
                &AbilityCost::Mana {
                    cost: ManaCost::generic(1)
                },
                "one age counter × cumulative upkeep {{1}} = {{1}}"
            );
        }
        other => panic!("expected UnlessPayment for Mystic Remora upkeep, got {other:?}"),
    }

    runner
        .state_mut()
        .players
        .iter_mut()
        .find(|p| p.id == P0)
        .unwrap()
        .mana_pool
        .mana = floating_colorless(1);

    runner
        .act(GameAction::PayUnlessCost { pay: true })
        .expect("paying upkeep must succeed when mana is available");

    assert_eq!(
        runner.state().objects[&remora].zone,
        Zone::Battlefield,
        "paying cumulative upkeep must keep Mystic Remora on the battlefield"
    );
}
