//! Issue #4786: Wrenn and Realmbreaker's -7 emblem must grant graveyard play/cast.

use engine::game::casting::graveyard_lands_playable_by_permission;
use engine::game::scenario::{GameScenario, P0};
use engine::game::zones::create_object;
use engine::types::ability::{AbilityCost, Effect, TargetFilter};
use engine::types::card_type::CoreType;
use engine::types::identifiers::CardId;
use engine::types::phase::Phase;
use engine::types::statics::{CastFrequency, StaticMode};
use engine::types::zones::Zone;

const WRENN_ORACLE: &str = "Lands you control have \"{T}: Add one mana of any color.\"\n\
+1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance, hexproof, and haste until your next turn. It's still a land.\n\
−2: Mill three cards. You may put a permanent card from among the milled cards into your hand.\n\
−7: You get an emblem with \"You may play lands and cast permanent spells from your graveyard.\"";

#[test]
fn wrenn_minus_seven_emblem_grants_graveyard_land_play() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let wrenn = scenario
        .add_creature(P0, "Wrenn and Realmbreaker", 0, 0)
        .from_oracle_text(WRENN_ORACLE)
        .id();

    let mut runner = scenario.build();
    {
        let wrenn_obj = runner.state_mut().objects.get_mut(&wrenn).unwrap();
        wrenn_obj.card_types.core_types = vec![CoreType::Planeswalker];
        wrenn_obj.loyalty = Some(10);
    }

    let minus_seven_index = runner.state().objects[&wrenn]
        .abilities
        .iter()
        .position(|ability| {
            matches!(
                ability.cost.as_ref(),
                Some(AbilityCost::Loyalty { amount: -7 })
            ) && matches!(ability.effect.as_ref(), Effect::CreateEmblem { .. })
        })
        .expect("Wrenn must expose a -7 CreateEmblem loyalty ability");

    runner.activate(wrenn, minus_seven_index).resolve();

    assert_eq!(
        runner.state().command_zone.len(),
        1,
        "activating -7 must create an emblem in the command zone"
    );
    let emblem_id = runner.state().command_zone[0];
    let emblem = &runner.state().objects[&emblem_id];
    assert!(emblem.is_emblem);

    let static_def = &emblem.static_definitions[0];
    assert!(
        matches!(
            static_def.mode,
            StaticMode::GraveyardCastPermission {
                frequency: CastFrequency::Unlimited,
                ..
            }
        ),
        "Wrenn emblem must install a graveyard play/cast permission static"
    );
    assert!(
        static_def.active_zones.contains(&Zone::Command),
        "emblem permission static must function from the command zone"
    );
    match &static_def.affected {
        Some(TargetFilter::Or { filters }) => assert_eq!(filters.len(), 2),
        other => panic!("expected combined land + permanent filter, got {other:?}"),
    }

    let forest = create_object(
        runner.state_mut(),
        CardId(9001),
        P0,
        "Forest".to_string(),
        Zone::Graveyard,
    );
    {
        let obj = runner.state_mut().objects.get_mut(&forest).unwrap();
        obj.card_types.core_types = vec![CoreType::Land];
    }

    let playable = graveyard_lands_playable_by_permission(runner.state(), P0);
    assert!(
        playable.iter().any(|(id, _)| *id == forest),
        "Forest in graveyard must be playable with Wrenn emblem active"
    );
}
