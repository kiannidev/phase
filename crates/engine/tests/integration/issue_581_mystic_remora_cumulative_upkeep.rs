//! Regression (issue #581): Mystic Remora cumulative upkeep must fire after
//! resolving a cast through the stack and after card-db rehydration rebuilds
//! a stale trigger index.

use engine::game::rehydrate_game_from_card_db;
use engine::game::scenario::GameScenario;
use engine::game::scenario_db::GameScenarioDbExt;
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

use crate::support::shared_card_db as load_db;

const P0: PlayerId = PlayerId(0);

fn floating_colorless(n: usize) -> Vec<ManaUnit> {
    (0..n)
        .map(|_| ManaUnit::new(ManaType::Colorless, ObjectId(0), false, vec![]))
        .collect()
}

fn add_mana(runner: &mut engine::game::scenario::GameRunner, mana: &[ManaType]) {
    let dummy = ObjectId(0);
    let pool = &mut runner
        .state_mut()
        .players
        .iter_mut()
        .find(|p| p.id == P0)
        .unwrap()
        .mana_pool;
    for m in mana {
        pool.add(ManaUnit::new(*m, dummy, false, vec![]));
    }
}

fn assert_has_cumulative_upkeep_trigger(
    state: &engine::types::game_state::GameState,
    remora: ObjectId,
) {
    assert!(
        state
            .objects
            .get(&remora)
            .unwrap()
            .trigger_definitions
            .as_slice()
            .iter()
            .any(|t| matches!(t.mode, TriggerMode::PayCumulativeUpkeep)),
        "Mystic Remora must carry a synthesized cumulative-upkeep trigger"
    );
}

fn assert_upkeep_unless_prompt(
    runner: &engine::game::scenario::GameRunner,
    remora: ObjectId,
    expected_generic: u32,
) {
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
                    cost: ManaCost::generic(expected_generic)
                },
                "age counter count × cumulative upkeep {{1}}"
            );
        }
        other => panic!("expected UnlessPayment for Mystic Remora upkeep, got {other:?}"),
    }
}

fn advance_to_upkeep_prompt(runner: &mut engine::game::scenario::GameRunner) {
    runner.advance_to_upkeep();
    runner.resolve_top();
}

#[test]
fn cast_pipeline_upkeep_adds_age_counter_and_prompts_payment() {
    let Some(db) = load_db() else {
        return;
    };

    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let remora = scenario.add_real_card(P0, "Mystic Remora", Zone::Hand, db);
    let mut runner = scenario.build();

    add_mana(&mut runner, &[ManaType::Blue]);
    runner.cast(remora).resolve();

    let remora_bf = runner
        .state()
        .objects
        .get(&remora)
        .expect("cast Mystic Remora")
        .zone;
    assert_eq!(remora_bf, Zone::Battlefield);
    assert_has_cumulative_upkeep_trigger(runner.state(), remora);

    // Cast in turn 1 main — cumulative upkeep first fires on turn 2 upkeep (CR 702.24).
    runner.state_mut().turn_number = 2;
    runner.state_mut().phase = Phase::Untap;
    runner.state_mut().active_player = P0;
    runner.state_mut().priority_player = P0;
    runner.state_mut().waiting_for = WaitingFor::Priority { player: P0 };
    runner.state_mut().stack.clear();

    advance_to_upkeep_prompt(&mut runner);
    assert_upkeep_unless_prompt(&runner, remora, 1);

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

#[test]
fn rehydrate_rebuilds_cumulative_upkeep_trigger_index() {
    let Some(db) = load_db() else {
        return;
    };

    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::Untap);
    let remora = scenario.add_real_card(P0, "Mystic Remora", Zone::Battlefield, db);
    let mut runner = scenario.build();

    assert_has_cumulative_upkeep_trigger(runner.state(), remora);

    // Simulate a stale derived index after an in-place card-db reload: object
    // definitions are intact but upkeep would not consult without rebuild.
    runner.state_mut().trigger_index.remove(remora);

    rehydrate_game_from_card_db(runner.state_mut(), db);

    advance_to_upkeep_prompt(&mut runner);
    assert_upkeep_unless_prompt(&runner, remora, 1);
}
