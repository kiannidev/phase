//! Regression for issue #2374: Fblthp, the Lost must draw 1 from hand and 2
//! when it entered or was cast from library.
//!
//! https://github.com/phase-rs/phase/issues/2374

use engine::parser::parse_oracle_text;
use engine::types::ability::{AbilityCondition, Effect, QuantityExpr};

const FBLTHP_ETB: &str = "When Fblthp enters, draw a card. If it entered from your library or was cast from your library, draw two cards instead.";

#[test]
fn fblthp_etb_parses_library_origin_instead_draw() {
    let parsed = parse_oracle_text(
        FBLTHP_ETB,
        "Fblthp, the Lost",
        &[],
        &["Creature".to_string()],
        &["Homunculus".to_string()],
    );
    let trigger = parsed
        .triggers
        .first()
        .expect("Fblthp must have an ETB trigger");
    let execute = trigger.execute.as_ref().expect("ETB trigger must execute");
    assert!(matches!(
        execute.effect.as_ref(),
        Effect::Draw {
            count: QuantityExpr::Fixed { value: 1 },
            ..
        }
    ));
    let instead = execute
        .sub_ability
        .as_ref()
        .expect("library-origin instead rider must be a sub_ability");
    assert!(matches!(
        instead.effect.as_ref(),
        Effect::Draw {
            count: QuantityExpr::Fixed { value: 2 },
            ..
        }
    ));
    assert!(matches!(
        instead.condition.as_ref(),
        Some(AbilityCondition::ConditionInstead { inner })
            if matches!(
                inner.as_ref(),
                AbilityCondition::Or { conditions }
                    if conditions.len() == 2
            )
    ));
}
