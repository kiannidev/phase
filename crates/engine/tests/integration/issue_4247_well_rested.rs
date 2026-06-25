//! Regression for issue #4247: Well Rested's granted untap trigger must be
//! controlled by the Aura's controller, not the enchanted creature's owner.
//!
//! Oracle text:
//!   Enchant creature
//!   Enchanted creature has "Whenever this creature becomes untapped, put two
//!   +1/+1 counters on it, then you gain 2 life and draw a card. This ability
//!   triggers only once each turn."
//!
//! CR 303.4e: an Aura's controller is separate from its enchanted permanent's
//! controller, so P0 must draw and gain life when P0's Well Rested is attached
//! to a creature P1 controls.

use engine::game::game_object::AttachTarget;
use engine::game::layers::evaluate_layers;
use engine::game::scenario::{GameScenario, P0, P1};
use engine::game::trigger_index::reindex_object_triggers;
use engine::game::triggers::{drain_order_triggers_with_identity, process_triggers};
use engine::types::events::GameEvent;
use engine::types::phase::Phase;

const WELL_RESTED_ORACLE: &str = "Enchant creature\nEnchanted creature has \
\"Whenever this creature becomes untapped, put two +1/+1 counters on it, \
then you gain 2 life and draw a card. This ability triggers only once each turn.\"";

#[test]
fn well_rested_granted_untap_trigger_routes_to_aura_controller() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    scenario.with_library_top(P0, &["Forest"]);

    let host = scenario.add_creature(P1, "Grizzly Bears", 2, 2).id();
    let well_rested = {
        let mut builder = scenario.add_creature(P0, "Well Rested", 0, 0);
        builder.as_enchantment();
        builder.with_subtypes(vec!["Aura"]);
        builder.from_oracle_text(WELL_RESTED_ORACLE);
        builder.id()
    };

    let mut runner = scenario.build();

    {
        let state = runner.state_mut();
        let aura_obj = state.objects.get_mut(&well_rested).unwrap();
        aura_obj.attached_to = Some(AttachTarget::Object(host));
        state
            .objects
            .get_mut(&host)
            .unwrap()
            .attachments
            .push(well_rested);
    }
    evaluate_layers(runner.state_mut());
    reindex_object_triggers(runner.state_mut(), host);

    runner.state_mut().objects.get_mut(&host).unwrap().tapped = true;

    let p0_life_before = runner.life(P0);
    let p1_life_before = runner.life(P1);
    let p0_hand_before = runner.state().players[0].hand.len();
    let p1_hand_before = runner.state().players[1].hand.len();

    let events = vec![GameEvent::PermanentUntapped { object_id: host }];
    process_triggers(runner.state_mut(), &events);
    drain_order_triggers_with_identity(runner.state_mut());
    runner.advance_until_stack_empty();

    assert_eq!(
        runner.life(P0),
        p0_life_before + 2,
        "Well Rested's controller (P0) must gain 2 life from the granted trigger"
    );
    assert_eq!(
        runner.state().players[0].hand.len(),
        p0_hand_before + 1,
        "Well Rested's controller (P0) must draw from the granted trigger"
    );
    assert_eq!(
        runner.life(P1),
        p1_life_before,
        "the enchanted creature's controller (P1) must not gain life"
    );
    assert_eq!(
        runner.state().players[1].hand.len(),
        p1_hand_before,
        "the enchanted creature's controller (P1) must not draw"
    );
}
