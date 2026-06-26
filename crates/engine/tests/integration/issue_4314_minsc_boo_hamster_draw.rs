//! Issue #4314 — Minsc & Boo [-2]: sacrificing a Hamster must draw X cards
//! after the intervening WhenYouDo damage step.

use engine::game::ability_utils::build_resolved_from_def;
use engine::game::effects::resolve_ability_chain;
use engine::game::scenario::{GameScenario, P0, P1};
use engine::parser::oracle_effect::parse_effect_chain;
use engine::types::ability::{AbilityKind, TargetRef};
use engine::types::actions::GameAction;
use engine::types::game_state::WaitingFor;
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const MINSC_MINUS_TWO: &str = "Sacrifice a creature. When you do, Minsc & Boo deals X damage to any target, where X is that creature's power. If the sacrificed creature was a Hamster, draw X cards.";

fn resolve_minsc_choices(
    runner: &mut engine::game::scenario::GameRunner,
    sacrifice: engine::types::identifiers::ObjectId,
    damage_target: engine::types::identifiers::ObjectId,
) {
    for _ in 0..30 {
        match runner.state().waiting_for.clone() {
            WaitingFor::EffectZoneChoice { .. } | WaitingFor::PayCost { .. } => {
                runner
                    .act(GameAction::SelectCards {
                        cards: vec![sacrifice],
                    })
                    .expect("sacrifice choice");
            }
            WaitingFor::TargetSelection { .. } | WaitingFor::TriggerTargetSelection { .. } => {
                runner
                    .act(GameAction::SelectTargets {
                        targets: vec![TargetRef::Object(damage_target)],
                    })
                    .expect("damage target");
            }
            WaitingFor::MultiTargetSelection { .. } => {
                runner
                    .act(GameAction::SelectTargets {
                        targets: vec![TargetRef::Object(damage_target)],
                    })
                    .expect("damage target");
            }
            WaitingFor::Priority { .. } if runner.state().stack.is_empty() => break,
            _ if runner.state().stack.is_empty() => break,
            _ => {
                runner.act(GameAction::PassPriority).ok();
            }
        }
    }
}

#[test]
fn minsc_hamster_sacrifice_draws_power_after_damage_step() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);

    let minsc = scenario
        .add_creature(P0, "Minsc & Boo", 0, 0)
        .from_oracle_text(MINSC_MINUS_TWO)
        .id();
    let boo = scenario
        .add_creature(P0, "Boo", 1, 1)
        .with_subtypes(vec!["Hamster"])
        .with_plus_counters(2)
        .id();
    let bear = scenario.add_creature(P1, "Bear", 2, 2).id();
    scenario.with_library_top(P0, &["L1", "L2", "L3", "L4", "L5", "L6"]);

    let mut runner = scenario.build();
    let hand_before = runner.state().players[P0.0 as usize].hand.len();
    let lib_before = runner.state().players[P0.0 as usize].library.len();

    let def = parse_effect_chain(MINSC_MINUS_TWO, AbilityKind::Activated);
    let ability = build_resolved_from_def(&def, minsc, P0);
    let mut events = Vec::new();
    resolve_ability_chain(runner.state_mut(), &ability, &mut events, 0).unwrap();
    resolve_minsc_choices(&mut runner, boo, bear);

    assert_eq!(runner.state().objects[&boo].zone, Zone::Graveyard);
    assert_eq!(
        runner.state().players[P0.0 as usize].hand.len(),
        hand_before + 3,
        "Hamster sacrifice must draw 3 (1/1 + two +1/+1 counters)"
    );
    assert_eq!(
        lib_before - runner.state().players[P0.0 as usize].library.len(),
        3
    );
}
