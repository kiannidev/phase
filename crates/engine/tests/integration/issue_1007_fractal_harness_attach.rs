//! Issue #1007 — Fractal Harness must attach to the Fractal token it creates on ETB.

use engine::game::game_object::AttachTarget;
use engine::game::scenario::{GameScenario, P0};
use engine::types::ability::{Effect, QuantityExpr, QuantityRef, TargetFilter};
use engine::types::counter::CounterType;
use engine::types::identifiers::ObjectId;
use engine::types::mana::{ManaCost, ManaCostShard, ManaType, ManaUnit};
use engine::types::phase::Phase;

const FRACTAL_HARNESS_ORACLE: &str = "When this Equipment enters, create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it and attach this Equipment to it.\nWhenever equipped creature attacks, double the number of +1/+1 counters on it.\nEquip {2}";

fn etb_attach_effect() -> Effect {
    let parsed = engine::parser::parse_oracle_text(
        FRACTAL_HARNESS_ORACLE,
        "Fractal Harness",
        &[],
        &["Artifact".to_string()],
        &["Equipment".to_string()],
    );
    let execute = parsed
        .triggers
        .iter()
        .find(|t| t.mode == engine::types::triggers::TriggerMode::ChangesZone)
        .and_then(|t| t.execute.as_ref())
        .expect("ETB trigger");
    let mut current: Option<&engine::types::ability::AbilityDefinition> = Some(execute);
    while let Some(ability) = current {
        if let Effect::Attach { .. } = ability.effect.as_ref() {
            return ability.effect.as_ref().clone();
        }
        current = ability.sub_ability.as_deref();
    }
    panic!("missing Attach in ETB chain: {:?}", execute.effect);
}

fn pool_for_x(x: u32) -> Vec<ManaUnit> {
    let mut pool = vec![
        ManaUnit::new(ManaType::Green, ObjectId(0), false, vec![]),
        ManaUnit::new(ManaType::Colorless, ObjectId(0), false, vec![]),
        ManaUnit::new(ManaType::Colorless, ObjectId(0), false, vec![]),
    ];
    for _ in 0..x {
        pool.push(ManaUnit::new(
            ManaType::Colorless,
            ObjectId(0),
            false,
            vec![],
        ));
    }
    pool
}

#[test]
fn fractal_harness_etb_token_enters_with_counters_and_attach_sub() {
    let parsed = engine::parser::parse_oracle_text(
        FRACTAL_HARNESS_ORACLE,
        "Fractal Harness",
        &[],
        &["Artifact".to_string()],
        &["Equipment".to_string()],
    );
    let execute = parsed
        .triggers
        .iter()
        .find(|t| t.mode == engine::types::triggers::TriggerMode::ChangesZone)
        .and_then(|t| t.execute.as_ref())
        .expect("ETB trigger");
    let Effect::Token {
        enter_with_counters,
        ..
    } = execute.effect.as_ref()
    else {
        panic!("ETB root must be Token, got {:?}", execute.effect);
    };
    assert_eq!(enter_with_counters.len(), 1);
    assert_eq!(enter_with_counters[0].0, CounterType::Plus1Plus1);
    assert!(
        matches!(
            enter_with_counters[0].1,
            QuantityExpr::Ref {
                qty: QuantityRef::CostXPaid
            }
        ) || matches!(
            enter_with_counters[0].1,
            QuantityExpr::Ref {
                qty: QuantityRef::Variable { .. }
            }
        ),
        "X paid at cast must bind enter_with_counters, got {:?}",
        enter_with_counters[0].1
    );
    let attach = execute
        .sub_ability
        .as_ref()
        .expect("attach must follow token creation directly");
    let Effect::Attach { attachment, target } = attach.effect.as_ref() else {
        panic!("expected Attach sub, got {:?}", attach.effect);
    };
    assert_eq!(*attachment, TargetFilter::SelfRef);
    assert_eq!(*target, TargetFilter::LastCreated);
    assert!(
        attach.sub_ability.is_none(),
        "PutCounter sibling must be folded into enter_with_counters"
    );
}

#[test]
fn fractal_harness_etb_attach_targets_last_created_token() {
    let Effect::Attach { attachment, target } = etb_attach_effect() else {
        panic!("expected Attach effect");
    };
    assert_eq!(attachment, TargetFilter::SelfRef);
    assert_eq!(
        target,
        TargetFilter::LastCreated,
        "post-token 'attach this Equipment to it' must target the created token"
    );
}

#[test]
fn fractal_harness_attaches_to_created_token_on_etb() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);
    scenario.with_mana_pool(P0, pool_for_x(3));

    let harness = scenario
        .add_creature_to_hand_from_oracle(P0, "Fractal Harness", 0, 0, FRACTAL_HARNESS_ORACLE)
        .as_artifact()
        .with_subtypes(vec!["Equipment"])
        .with_mana_cost(ManaCost::Cost {
            shards: vec![ManaCostShard::X, ManaCostShard::Green],
            generic: 2,
        })
        .id();

    let mut runner = scenario.build();
    runner.cast(harness).x(3).resolve();

    let harness_obj = runner.state().objects.get(&harness).expect("harness");
    assert_eq!(
        harness_obj.zone,
        engine::types::zones::Zone::Battlefield,
        "Fractal Harness should enter the battlefield"
    );

    assert_eq!(
        harness_obj.counters.get(&CounterType::Plus1Plus1).copied(),
        None,
        "Fractal Harness itself should not receive the X counters"
    );

    let attached_host = harness_obj
        .attached_to
        .as_ref()
        .and_then(|t| match t {
            AttachTarget::Object(id) => Some(*id),
            AttachTarget::Player(_) => None,
        })
        .expect("Fractal Harness should attach to the created token on ETB");

    let token = runner
        .state()
        .objects
        .get(&attached_host)
        .expect("created token");
    assert!(
        token.is_token,
        "attachment host should be the created token"
    );
    assert!(
        token.card_types.subtypes.iter().any(|s| s == "Fractal"),
        "host should be a Fractal token"
    );
    assert!(
        token.attachments.contains(&harness),
        "token should list Fractal Harness as attached equipment"
    );
    assert_eq!(
        token.counters.get(&CounterType::Plus1Plus1).copied(),
        Some(3),
        "token should receive X +1/+1 counters before attachment"
    );
}
