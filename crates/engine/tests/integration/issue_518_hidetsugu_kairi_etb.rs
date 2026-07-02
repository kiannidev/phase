//! Issue #518: Hidetsugu and Kairi ETB draws three, then puts two hand cards on top.

use engine::game::scenario::{GameScenario, P0};
use engine::types::actions::GameAction;
use engine::types::game_state::WaitingFor;
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const HIDETSUGU_AND_KAIRI_ORACLE: &str = "\
Flying\n\
When Hidetsugu and Kairi enters, draw three cards, then put two cards from your hand on top of your library in any order.\n\
When Hidetsugu and Kairi dies, exile the top card of your library. Target opponent loses life equal to its mana value. If it's an instant or sorcery card, you may cast it without paying its mana cost.";

#[test]
fn hidetsugu_and_kairi_etb_draws_then_puts_two_hand_cards_on_library_top() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    scenario.with_library_top(P0, &["Draw A", "Draw B", "Draw C"]);
    scenario.with_library_top(P0, &["Keep One", "Keep Two", "Keep Three", "Keep Four"]);

    let hk = scenario
        .add_creature_to_hand_from_oracle(
            P0,
            "Hidetsugu and Kairi",
            5,
            4,
            HIDETSUGU_AND_KAIRI_ORACLE,
        )
        .id();

    let mut runner = scenario.build();
    let hand_before = runner.state().players[0].hand.len();
    let library_before = runner.state().players[0].library.len();

    runner.cast(hk).resolve();

    for _ in 0..48 {
        match runner.state().waiting_for.clone() {
            WaitingFor::EffectZoneChoice { .. } => {
                let hand: Vec<_> = runner.state().players[0].hand.iter().copied().collect();
                runner
                    .act(GameAction::SelectCards {
                        cards: hand.iter().take(2).copied().collect(),
                    })
                    .expect("choose two hand cards for library top");
            }
            WaitingFor::Priority { .. } if runner.state().stack.is_empty() => break,
            _ => {
                runner.act(GameAction::PassPriority).expect("pass");
            }
        }
    }
    runner.advance_until_stack_empty();

    let state = runner.state();
    assert_eq!(
        state.players[0].hand.len(),
        hand_before - 1 + 3 - 2,
        "ETB must draw three and put two from hand on top"
    );
    assert_eq!(
        state.players[0].library.len(),
        library_before - 3 + 2,
        "library loses three drawn cards and gains two back on top"
    );
    assert_eq!(state.objects[&hk].zone, Zone::Battlefield);
}
