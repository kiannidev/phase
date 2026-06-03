use crate::types::ability::{Effect, EffectError, QuantityExpr, ResolvedAbility, TargetFilter};
use crate::types::events::GameEvent;
use crate::types::game_state::GameState;

/// CR 702.178a: When a creature with double team attacks, its controller creates
/// a tapped and attacking token copy of that creature.
pub fn resolve(
    state: &mut GameState,
    ability: &ResolvedAbility,
    events: &mut Vec<GameEvent>,
) -> Result<(), EffectError> {
    if !matches!(ability.effect, Effect::DoubleTeam) {
        return Err(EffectError::MissingParam("DoubleTeam".to_string()));
    }

    let copy_effect = Effect::CopyTokenOf {
        target: TargetFilter::SelfRef,
        owner: TargetFilter::Controller,
        source_filter: None,
        enters_attacking: true,
        tapped: true,
        count: QuantityExpr::Fixed { value: 1 },
        extra_keywords: vec![],
        additional_modifications: vec![],
    };
    let copy_ability =
        ResolvedAbility::new(copy_effect, vec![], ability.source_id, ability.controller);
    crate::game::effects::token_copy::resolve(state, &copy_ability, events)?;

    events.push(GameEvent::EffectResolved {
        kind: crate::types::ability::EffectKind::DoubleTeam,
        source_id: ability.source_id,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::database::synthesis::synthesize_all;
    use crate::game::combat::AttackTarget;
    use crate::game::printed_cards::apply_card_face_to_object;
    use crate::game::zones::create_object;
    use crate::types::ability::PtValue;
    use crate::types::actions::GameAction;
    use crate::types::card::CardFace;
    use crate::types::card_type::CoreType;
    use crate::types::format::FormatConfig;
    use crate::types::game_state::{GameState, WaitingFor};
    use crate::types::identifiers::{CardId, ObjectId};
    use crate::types::keywords::Keyword;
    use crate::types::phase::Phase;
    use crate::types::player::PlayerId;
    use crate::types::zones::Zone;

    fn double_team_face() -> CardFace {
        let mut face = CardFace {
            name: "Double Team Bear".to_string(),
            power: Some(PtValue::Fixed(2)),
            toughness: Some(PtValue::Fixed(2)),
            keywords: vec![Keyword::DoubleTeam],
            ..CardFace::default()
        };
        face.card_type.core_types.push(CoreType::Creature);
        synthesize_all(&mut face);
        face
    }

    fn setup() -> (GameState, ObjectId) {
        let face = double_team_face();
        let mut state = GameState::new(FormatConfig::standard(), 2, 42);
        state.turn_number = 2;
        state.phase = Phase::DeclareAttackers;
        state.active_player = PlayerId(0);
        state.priority_player = PlayerId(0);
        state.waiting_for = WaitingFor::DeclareAttackers {
            player: PlayerId(0),
            valid_attacker_ids: vec![],
            valid_attack_targets: vec![],
        };

        let card_id = CardId(state.next_object_id);
        let attacker_id = create_object(
            &mut state,
            card_id,
            PlayerId(0),
            face.name.clone(),
            Zone::Battlefield,
        );
        {
            let attacker = state.objects.get_mut(&attacker_id).unwrap();
            apply_card_face_to_object(attacker, &face);
            attacker.entered_battlefield_turn = Some(1);
        }
        (state, attacker_id)
    }

    #[test]
    fn double_team_attack_creates_tapped_attacking_copy() {
        let face = double_team_face();
        let (mut state, attacker_id) = setup();

        crate::game::engine::apply_as_current(
            &mut state,
            GameAction::DeclareAttackers {
                attacks: vec![(attacker_id, AttackTarget::Player(PlayerId(1)))],
            },
        )
        .expect("declare attacker");

        assert_eq!(
            state.stack.len(),
            1,
            "double team attack trigger should be on the stack"
        );

        let mut events = Vec::new();
        crate::game::stack::resolve_top(&mut state, &mut events);

        let tokens: Vec<_> = state
            .objects
            .iter()
            .filter_map(|(id, obj)| {
                (obj.is_token && obj.name == face.name && obj.zone == Zone::Battlefield)
                    .then_some(*id)
            })
            .collect();
        assert_eq!(tokens.len(), 1, "one tapped attacking copy");
        let token_id = tokens[0];
        assert!(state.objects.get(&token_id).unwrap().tapped);

        let combat = state.combat.as_ref().expect("combat active");
        let token_attacker = combat
            .attackers
            .iter()
            .find(|a| a.object_id == token_id)
            .expect("copy is attacking");
        assert_eq!(token_attacker.defending_player, PlayerId(1));
        assert_eq!(
            token_attacker.attack_target,
            AttackTarget::Player(PlayerId(1))
        );
        assert!(
            combat.attackers.iter().any(|a| a.object_id == attacker_id),
            "original attacker remains in combat"
        );
    }
}
