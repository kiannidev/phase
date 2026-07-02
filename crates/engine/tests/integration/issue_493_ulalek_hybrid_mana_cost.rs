//! Issue #493: Ulalek's {C/W}{C/U}{C/B}{C/R}{C/G} cost must be paid when casting.

use engine::game::scenario::{GameScenario, P0};
use engine::types::actions::GameAction;
use engine::types::game_state::CastPaymentMode;
use engine::types::mana::{ManaCost, ManaCostShard, ManaType, ManaUnit};
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const ULALEK_ORACLE: &str = "\
Devoid (This card has no color.)\n\
Whenever you cast an Eldrazi spell, you may pay {C}{C}. If you do, copy all spells you control, then copy all other activated and triggered abilities you control. You may choose new targets for the copies. (Mana abilities can't be copied.)";

fn ulalek_mana_cost() -> ManaCost {
    ManaCost::Cost {
        shards: vec![
            ManaCostShard::ColorlessWhite,
            ManaCostShard::ColorlessBlue,
            ManaCostShard::ColorlessBlack,
            ManaCostShard::ColorlessRed,
            ManaCostShard::ColorlessGreen,
        ],
        generic: 0,
    }
}

#[test]
fn ulalek_cast_requires_paying_colorless_hybrid_mana() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let ulalek = scenario
        .add_creature_to_hand_from_oracle(P0, "Ulalek, Fused Atrocity", 7, 5, ULALEK_ORACLE)
        .with_mana_cost(ulalek_mana_cost())
        .id();

    let mut runner = scenario.build();
    for color in [
        ManaType::White,
        ManaType::Blue,
        ManaType::Black,
        ManaType::Red,
        ManaType::Green,
    ] {
        runner.state_mut().players[0]
            .mana_pool
            .add(ManaUnit::new(color, ulalek, false, vec![]));
    }

    runner.cast(ulalek).resolve();
    runner.advance_until_stack_empty();

    assert_eq!(
        runner.state().objects[&ulalek].zone,
        Zone::Battlefield,
        "Ulalek must resolve after paying hybrid mana"
    );
    assert_eq!(
        runner.state().players[0].mana_pool.total(),
        0,
        "all five hybrid shards must be paid"
    );
}

#[test]
fn ulalek_cast_without_mana_is_rejected() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let ulalek = scenario
        .add_creature_to_hand_from_oracle(P0, "Ulalek, Fused Atrocity", 7, 5, ULALEK_ORACLE)
        .with_mana_cost(ulalek_mana_cost())
        .id();

    let mut runner = scenario.build();
    let card_id = runner.state().objects[&ulalek].card_id;
    let err = runner
        .act(GameAction::CastSpell {
            object_id: ulalek,
            card_id,
            targets: vec![],
            payment_mode: CastPaymentMode::Auto,
        })
        .expect_err("casting Ulalek without mana must fail");
    assert!(
        matches!(err, engine::game::EngineError::ActionNotAllowed(_)),
        "expected ActionNotAllowed, got {err:?}"
    );
}
