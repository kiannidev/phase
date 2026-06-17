//! [#2877](https://github.com/phase-rs/phase/issues/2877): a */* CDA creature
//! (e.g. Lumra, Bellow of the Woods) must retain its characteristic-defining
//! power/toughness while phased out instead of staying at base 0/0.

use engine::game::game_object::PhaseOutCause;
use engine::game::layers::{evaluate_layers, flush_layers};
use engine::game::phasing::phase_out_object;
use engine::game::sba::check_state_based_actions;
use engine::game::zones::create_object;
use engine::types::ability::{ContinuousModification, StaticDefinition, TargetFilter};
use engine::types::card_type::CoreType;
use engine::types::game_state::GameState;
use engine::types::identifiers::{CardId, ObjectId};
use engine::types::player::PlayerId;
use engine::types::zones::Zone;

fn setup_land(state: &mut GameState, controller: PlayerId) -> ObjectId {
    let id = create_object(
        state,
        CardId(1),
        controller,
        "Forest".to_string(),
        Zone::Battlefield,
    );
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.card_types.core_types = vec![CoreType::Land];
        obj.base_card_types = obj.card_types.clone();
    }
    id
}

fn setup_star_star_cda_creature(state: &mut GameState, pt: i32, controller: PlayerId) -> ObjectId {
    let id = create_object(
        state,
        CardId(2),
        controller,
        "Lumra Stand-in".to_string(),
        Zone::Battlefield,
    );
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.card_types.core_types = vec![CoreType::Creature];
        obj.base_card_types = obj.card_types.clone();
        obj.power = Some(0);
        obj.toughness = Some(0);
        obj.base_power = Some(0);
        obj.base_toughness = Some(0);

        let def = StaticDefinition::continuous()
            .affected(TargetFilter::SelfRef)
            .cda()
            .modifications(vec![
                ContinuousModification::SetPower { value: pt },
                ContinuousModification::SetToughness { value: pt },
            ]);
        obj.static_definitions = vec![def.clone()].into();
        obj.base_static_definitions = std::sync::Arc::new(vec![def]);
    }
    id
}

#[test]
fn phased_out_star_star_cda_creature_keeps_cda_power_toughness() {
    let mut state = GameState::new_two_player(42);
    let controller = PlayerId(0);
    for _ in 0..3 {
        setup_land(&mut state, controller);
    }
    let creature = setup_star_star_cda_creature(&mut state, 3, controller);

    state.layers_dirty.mark_full();
    evaluate_layers(&mut state);
    assert_eq!(state.objects[&creature].power, Some(3));
    assert_eq!(state.objects[&creature].toughness, Some(3));

    let mut events = Vec::new();
    phase_out_object(&mut state, creature, PhaseOutCause::Directly, &mut events);
    assert!(state.objects[&creature].is_phased_out());

    // Simulate the post-reset state a */* creature can be left in when excluded
    // from the main layer pass: base 0/0 with no CDA re-application.
    {
        let obj = state.objects.get_mut(&creature).unwrap();
        obj.power = Some(0);
        obj.toughness = Some(0);
    }

    state.layers_dirty.mark_full();
    flush_layers(&mut state);

    assert_eq!(
        state.objects[&creature].power,
        Some(3),
        "phased-out */* CDA creature must re-derive P/T from its own CDA"
    );
    assert_eq!(state.objects[&creature].toughness, Some(3));

    check_state_based_actions(&mut state, &mut events);
    assert_eq!(
        state.objects[&creature].zone,
        Zone::Battlefield,
        "phased-out creature must not die to CR 704.5f while its CDA P/T applies"
    );
}
