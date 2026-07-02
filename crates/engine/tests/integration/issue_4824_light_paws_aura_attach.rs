//! Issue #4824: Light-Paws searched Auras must attach to Light-Paws, not a chosen creature.

use engine::parser::oracle::parse_oracle_text;
use engine::types::ability::{Effect, TargetFilter};

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
