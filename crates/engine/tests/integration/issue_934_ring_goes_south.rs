//! Issue #934 — The Ring Goes South must reveal until X land cards and put
//! those lands onto the battlefield tapped.
//!
//! https://github.com/phase-rs/phase/issues/934

use engine::game::scenario::{GameScenario, P0};
use engine::parser::oracle::parse_oracle_text;
use engine::types::ability::{Effect, TargetFilter, TypedFilter, TypeFilter};
use engine::types::actions::GameAction;
use engine::types::card_type::{CoreType, Supertype};
use engine::types::game_state::WaitingFor;
use engine::types::mana::ManaCost;
use engine::types::phase::Phase;
use engine::types::zones::Zone;

const RING_GOES_SOUTH_ORACLE: &str = "The Ring tempts you. Then reveal cards from the top of your library until you reveal X land cards, where X is the number of legendary creatures you control. Put those land cards onto the battlefield tapped and the rest on the bottom of your library in a random order.";

#[test]
fn ring_goes_south_parses_reveal_until_lands_to_battlefield_tapped() {
    let parsed = parse_oracle_text(
        RING_GOES_SOUTH_ORACLE,
        "The Ring Goes South",
        &[],
        &["Sorcery".to_string()],
        &[],
    );
    let spell = parsed.abilities.first().expect("sorcery spell ability");
    let reveal = spell
        .sub_ability
        .as_ref()
        .expect("RingTemptsYou chains RevealUntil");
    match reveal.effect.as_ref() {
        Effect::RevealUntil {
            filter: TargetFilter::Typed(TypedFilter { type_filters, .. }),
            kept_destination,
            rest_destination,
            enter_tapped,
            ..
        } => {
            assert!(type_filters.contains(&TypeFilter::Land));
            assert_eq!(*kept_destination, Zone::Battlefield);
            assert_eq!(*rest_destination, Zone::Library);
            assert!(enter_tapped.is_tapped());
        }
        other => panic!("expected RevealUntil, got {other:?}"),
    }
}

#[test]
fn ring_goes_south_puts_revealed_lands_onto_battlefield_tapped() {
    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);

    let legendary = scenario
        .add_creature(P0, "Legendary Hero", 2, 2)
        .id();

    let island = scenario.add_card_to_library_top(P0, "Island");
    let forest = scenario.add_card_to_library_top(P0, "Forest");

    let spell = scenario
        .add_spell_to_hand_from_oracle(P0, "The Ring Goes South", false, RING_GOES_SOUTH_ORACLE)
        .with_mana_cost(ManaCost::generic(0))
        .id();

    let mut runner = scenario.build();
    {
        let obj = runner.state_mut().objects.get_mut(&legendary).unwrap();
        obj.card_types.supertypes.push(Supertype::Legendary);
        obj.base_card_types = obj.card_types.clone();
    }
    {
        let island_obj = runner.state_mut().objects.get_mut(&island).unwrap();
        island_obj.card_types.core_types.push(CoreType::Instant);
        island_obj.base_card_types = island_obj.card_types.clone();
    }
    {
        let forest_obj = runner.state_mut().objects.get_mut(&forest).unwrap();
        forest_obj.card_types.core_types.push(CoreType::Land);
        forest_obj.base_card_types = forest_obj.card_types.clone();
    }

    runner.cast(spell).resolve();

    let mut resolved = false;
    for _ in 0..96 {
        match runner.state().waiting_for.clone() {
            WaitingFor::ChooseRingBearer { candidates, .. } => {
                runner
                    .act(GameAction::ChooseRingBearer {
                        target: candidates[0],
                    })
                    .expect("choose ring bearer");
            }
            WaitingFor::Priority { .. } if runner.state().stack.is_empty() => {
                resolved = true;
                break;
            }
            _ => {
                runner.act(GameAction::PassPriority).ok();
            }
        }
    }

    assert!(resolved, "Ring Goes South must resolve; waiting_for={:?}", runner.state().waiting_for);
    assert!(
        runner.state().objects.values().any(|obj| {
            obj.zone == Zone::Battlefield
                && obj.name == "Forest"
                && obj.tapped
        }),
        "revealed Forest must enter the battlefield tapped; bf={:?}",
        runner
            .state()
            .objects
            .values()
            .filter(|obj| obj.zone == Zone::Battlefield)
            .map(|obj| (&obj.name, obj.tapped))
            .collect::<Vec<_>>()
    );
    assert!(
        runner.state().players[0]
            .library
            .iter()
            .any(|id| runner.state().objects[id].name == "Island"),
        "nonland cards revealed before the land must go to the bottom of the library"
    );
}
