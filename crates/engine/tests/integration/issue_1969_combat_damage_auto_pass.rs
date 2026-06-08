//! Issue #1969 — combat damage must be dealt even when the active player is in an
//! UntilEndOfTurn auto-pass session ("pass to end step" must not skip damage).

use engine::game::combat::AttackTarget;
use engine::game::scenario::{GameScenario, P0, P1};
use engine::types::actions::GameAction;
use engine::types::game_state::{AutoPassRequest, WaitingFor};
use engine::types::phase::Phase;

#[test]
fn until_end_of_turn_auto_pass_still_deals_combat_damage() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let attacker_id = scenario.add_creature(P0, "Bear", 3, 3).id();
    let mut runner = scenario.build();

    // Reach declare attackers without auto-pass interfering.
    runner.pass_both_players();
    runner
        .act(GameAction::DeclareAttackers {
            attacks: vec![(attacker_id, AttackTarget::Player(P1))],
            bands: vec![],
        })
        .expect("declare attackers");
    if matches!(runner.state().waiting_for, WaitingFor::Priority { .. }) {
        runner.pass_both_players();
    }
    if matches!(
        runner.state().waiting_for,
        WaitingFor::DeclareBlockers { .. }
    ) {
        runner
            .act(GameAction::DeclareBlockers {
                assignments: vec![],
            })
            .expect("declare no blockers");
    }

    // Enable "pass to end step" before the declare-blockers priority window drains.
    runner
        .act(GameAction::SetAutoPass {
            mode: AutoPassRequest::UntilEndOfTurn,
        })
        .expect("enable pass-to-end");

    // Auto-pass through combat damage and the rest of combat.
    for _ in 0..40 {
        if runner.state().phase == Phase::PostCombatMain {
            break;
        }
        match &runner.state().waiting_for {
            WaitingFor::Priority { .. } => {
                let _ = runner.act(GameAction::PassPriority);
            }
            _ => break,
        }
    }

    let p1_life = runner
        .state()
        .players
        .iter()
        .find(|p| p.id == P1)
        .unwrap()
        .life;
    assert_eq!(
        p1_life, 17,
        "defender must take 3 combat damage even under UntilEndOfTurn auto-pass; \
         phase={:?}, waiting_for={:?}",
        runner.state().phase,
        runner.state().waiting_for
    );
}
