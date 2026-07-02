//! Issue #515: Playing Cabal Stronghold must not surface a spurious optional effect.

use engine::game::scenario::{GameScenario, P0};
use engine::types::actions::GameAction;
use engine::types::game_state::WaitingFor;
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const CABAL_STRONGHOLD_ORACLE: &str =
    "{T}: Add {C}.\n{3}, {T}: Add {B} for each basic Swamp you control.";

#[test]
fn cabal_stronghold_play_land_no_optional_prompt() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let land = scenario
        .add_land_to_hand(P0, "Cabal Stronghold")
        .from_oracle_text(CABAL_STRONGHOLD_ORACLE)
        .id();
    let mut runner = scenario.build();

    let card_id = runner.state().objects[&land].card_id;
    runner
        .act(GameAction::PlayLand {
            object_id: land,
            card_id,
        })
        .expect("play land");

    assert_eq!(runner.state().objects[&land].zone, Zone::Battlefield);
    assert!(
        !matches!(
            runner.state().waiting_for,
            WaitingFor::OptionalEffectChoice { .. }
        ),
        "playing Cabal Stronghold must not prompt OptionalEffectChoice, got {:?}",
        runner.state().waiting_for
    );
}
