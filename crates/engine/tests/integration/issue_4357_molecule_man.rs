//! Regression for issue #4357 — Molecule Man must offer miracle {0} when the
//! controller draws a nonland card as their first draw of the turn.
//!
//! https://github.com/phase-rs/phase/issues/4357

use engine::game::effects::draw::resolve as resolve_draw;
use engine::game::scenario::{GameScenario, P0};
use engine::types::ability::{Effect, QuantityExpr, ResolvedAbility, TargetFilter};
use engine::types::identifiers::ObjectId;
use engine::types::mana::ManaCost;
use engine::types::phase::Phase;

const MOLECULE_MAN: &str = "Nonland cards in your hand have miracle {0}. (You may cast a card for its miracle cost when you draw it if it's the first card you drew this turn.)";

#[test]
fn molecule_man_offers_miracle_on_first_nonland_draw() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    scenario
        .add_creature(P0, "Molecule Man", 5, 5)
        .from_oracle_text(MOLECULE_MAN);
    scenario
        .add_spell_to_library_top(P0, "Shock", false)
        .with_mana_cost(ManaCost::Cost {
            shards: vec![],
            generic: 1,
        });
    scenario.add_card_to_library_top(P0, "Island");

    let mut runner = scenario.build();

    let draw = ResolvedAbility::new(
        Effect::Draw {
            count: QuantityExpr::Fixed { value: 1 },
            target: TargetFilter::Controller,
        },
        Vec::new(),
        ObjectId(0),
        P0,
    );
    let mut events = Vec::new();
    resolve_draw(runner.state_mut(), &draw, &mut events).expect("draw resolves");

    assert_eq!(
        runner.state().pending_miracle_offers.len(),
        1,
        "first draw of a nonland must queue a miracle offer under Molecule Man"
    );
    assert!(
        runner.state().pending_miracle_offers[0]
            .cost
            .is_without_paying_mana(),
        "miracle {{0}} must be a zero-cost alternative, got {:?}",
        runner.state().pending_miracle_offers[0].cost
    );
}
