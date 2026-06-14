//! Issue #1022 — Savai Triome cycling from the battlefield must discard the land
//! and draw a card.

use engine::game::casting::can_activate_ability_now;
use engine::game::scenario::{GameScenario, P0};
use engine::types::ability::AbilityTag;
use engine::types::actions::GameAction;
use engine::types::identifiers::ObjectId;
use engine::types::mana::{ManaType, ManaUnit};
use engine::types::phase::Phase;
use engine::types::zones::Zone;

#[test]
fn savai_triome_battlefield_cycling_draws() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    scenario.with_library_top(P0, &["Cycled Draw"]);
    scenario.with_mana_pool(
        P0,
        vec![ManaUnit::new(ManaType::Colorless, ObjectId(9_999), false, vec![]); 3],
    );

    let triome = scenario
        .add_land_to_hand(P0, "Savai Triome")
        .from_oracle_text(
            "({T}: Add {R}, {W}, or {B}.)\nThis land enters tapped.\nCycling {3}",
        )
        .id();

    let mut runner = scenario.build();
    let card_id = runner.state().objects[&triome].card_id;
    let library_before = runner.state().players[0].library.len();

    runner
        .act(GameAction::PlayLand {
            object_id: triome,
            card_id,
        })
        .expect("play Savai Triome");

    assert_eq!(runner.state().objects[&triome].zone, Zone::Battlefield);
    assert!(
        runner.state().players[0].hand.is_empty(),
        "triome must leave hand when played"
    );

    let cycling_index = runner.state().objects[&triome]
        .abilities
        .iter()
        .position(|ability| ability.ability_tag == Some(AbilityTag::Cycling))
        .expect("synthesized cycling ability");

    assert!(
        can_activate_ability_now(runner.state(), P0, triome, cycling_index),
        "cycling must be legal from the battlefield"
    );

    runner
        .act(GameAction::ActivateAbility {
            source_id: triome,
            ability_index: cycling_index,
        })
        .expect("activate cycling");

    runner.advance_until_stack_empty();

    assert_eq!(
        runner.state().objects[&triome].zone,
        Zone::Graveyard,
        "cycling must discard the triome"
    );
    assert_eq!(
        runner.state().players[0].hand.len(),
        1,
        "cycling must draw exactly one card"
    );
    assert_eq!(
        runner.state().players[0].library.len(),
        library_before - 1,
        "cycling must draw from the library"
    );
}
