//! Issue #4824: Light-Paws searched Auras must attach to Light-Paws, not a chosen creature.

use engine::game::game_object::AttachTarget;
use engine::game::scenario::{GameScenario, P0};
use engine::parser::oracle::parse_oracle_text;
use engine::types::ability::{Effect, TargetFilter, TypedFilter};
use engine::types::game_state::WaitingFor;
use engine::types::identifiers::ObjectId;
use engine::types::keywords::Keyword;
use engine::types::mana::{ManaCost, ManaCostShard, ManaType, ManaUnit};
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const LIGHT_PAWS_ORACLE: &str =
    "Whenever an Aura you control enters, if you cast it, you may search your library for an Aura card with mana value less than or equal to that Aura and with a different name than each Aura you control, put that card onto the battlefield attached to Light-Paws, then shuffle.";

#[test]
fn light_paws_oracle_search_attach_host_parses_as_self_ref() {
    let parsed = parse_oracle_text(
        LIGHT_PAWS_ORACLE,
        "Light-Paws, Emperor's Voice",
        &[],
        &["Creature".to_string()],
        &[],
    );
    let trigger = parsed.triggers.first().expect("trigger");
    let execute = trigger.execute.as_ref().expect("execute");
    let sub = execute.sub_ability.as_ref().expect("change zone sub");
    let attach = sub
        .sub_ability
        .as_ref()
        .expect("attach sub")
        .effect
        .as_ref();
    match attach {
        Effect::Attach { target, .. } => {
            assert_eq!(
                target,
                &TargetFilter::SelfRef,
                "search put-step must attach to the ability source (~)"
            );
        }
        other => panic!("expected Attach sub, got {other:?}"),
    }
}

#[test]
fn light_paws_searched_aura_enters_attached_without_host_prompt() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    let light_paws = scenario
        .add_creature_from_oracle(P0, "Light-Paws, Emperor's Voice", 1, 2, LIGHT_PAWS_ORACLE)
        .id();

    let searched_aura = scenario
        .add_spell_to_library_top(P0, "Search Aura Two", false)
        .as_enchantment()
        .with_subtypes(vec!["Aura"])
        .with_keyword(Keyword::Enchant(TargetFilter::Typed(
            TypedFilter::creature(),
        )))
        .with_mana_cost(ManaCost::Cost {
            generic: 1,
            shards: vec![ManaCostShard::White],
        })
        .id();

    let cast_aura = scenario
        .add_spell_to_hand(P0, "Cast Aura One", false)
        .as_enchantment()
        .with_subtypes(vec!["Aura"])
        .with_keyword(Keyword::Enchant(TargetFilter::Typed(
            TypedFilter::creature(),
        )))
        .with_mana_cost(ManaCost::Cost {
            generic: 1,
            shards: vec![ManaCostShard::White],
        })
        .id();

    scenario.with_mana_pool(
        P0,
        vec![
            ManaUnit::new(ManaType::White, ObjectId(9_998), false, vec![]),
            ManaUnit::new(ManaType::White, ObjectId(9_999), false, vec![]),
        ],
    );

    let mut runner = scenario.build();

    runner
        .cast(cast_aura)
        .target_object(light_paws)
        .accept_optional()
        .search_first_legal()
        .resolve();

    assert_eq!(
        runner.state().objects[&searched_aura].zone,
        Zone::Battlefield,
        "searched Aura must enter the battlefield from the library search"
    );
    assert_eq!(
        runner.state().objects[&searched_aura].attached_to,
        Some(AttachTarget::Object(light_paws)),
        "searched Aura must attach to Light-Paws without a host-choice prompt"
    );
    assert!(
        !matches!(
            runner.state().waiting_for,
            WaitingFor::ReturnAsAuraTarget { .. } | WaitingFor::TargetSelection { .. }
        ),
        "search put must not surface an Aura host prompt, got {:?}",
        runner.state().waiting_for
    );
}
