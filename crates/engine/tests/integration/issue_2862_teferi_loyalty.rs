//! Regression for issue #2862: Teferi, Time Raveler must lose loyalty from
//! combat damage and pay loyalty costs for [-3] even when the loyalty counter
//! map was not seeded at entry.
//!
//! https://github.com/phase-rs/phase/issues/2862

use engine::game::deck_loading::create_object_from_card_face;
use engine::game::scenario::{GameScenario, P0};
use engine::types::counter::CounterType;
use engine::types::phase::Phase;
use engine::types::zones::Zone;

fn issue_2862_db() -> &'static engine::database::card_db::CardDatabase {
    static DB: std::sync::OnceLock<engine::database::card_db::CardDatabase> =
        std::sync::OnceLock::new();
    DB.get_or_init(|| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/issue_2862_cards.json");
        engine::database::card_db::CardDatabase::from_export(&path)
            .expect("issue_2862_cards.json fixture must load")
    })
}

fn place_teferi_without_loyalty_counters(
    state: &mut engine::types::game_state::GameState,
    db: &engine::database::card_db::CardDatabase,
) -> engine::types::identifiers::ObjectId {
    let face = db
        .get_face_by_name("Teferi, Time Raveler")
        .expect("Teferi fixture");
    let id = create_object_from_card_face(state, face, P0);
    engine::game::zones::remove_from_zone(state, id, Zone::Library, P0);
    engine::game::zones::add_to_zone(state, id, Zone::Battlefield, P0);
    let obj = state.objects.get_mut(&id).unwrap();
    obj.zone = Zone::Battlefield;
    // Simulate objects that carry the printed loyalty characteristic but never
    // received intrinsic ETB loyalty counters (stale rehydrate / debug placement).
    obj.counters.clear();
    assert_eq!(obj.loyalty, Some(4));
    id
}

#[test]
fn issue_2862_teferi_minus_three_pays_loyalty_cost_without_counter_map() {
    let db = issue_2862_db();

    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);

    let mut runner = scenario.build();
    engine::game::rehydrate_game_from_card_db(runner.state_mut(), db);
    let teferi = place_teferi_without_loyalty_counters(runner.state_mut(), db);

    let mut events = Vec::new();
    engine::game::casting::pay_ability_cost(
        runner.state_mut(),
        P0,
        teferi,
        &engine::types::ability::AbilityCost::Loyalty { amount: -3 },
        &mut events,
    )
    .expect("pay [-3] loyalty cost");

    assert_eq!(
        runner.state().objects[&teferi].loyalty,
        Some(1),
        "activating [-3] must remove 3 loyalty (4 → 1)"
    );
    assert_eq!(
        runner
            .state()
            .objects[&teferi]
            .counters
            .get(&CounterType::Loyalty)
            .copied(),
        Some(1),
        "loyalty counter map must track the paid cost"
    );
}

#[test]
fn issue_2862_teferi_combat_damage_removes_loyalty_without_counter_map() {
    let db = issue_2862_db();

    let mut scenario = GameScenario::new();
    scenario.at_phase(Phase::PreCombatMain);

    let mut runner = scenario.build();
    engine::game::rehydrate_game_from_card_db(runner.state_mut(), db);
    let teferi = place_teferi_without_loyalty_counters(runner.state_mut(), db);

    let mut events = Vec::new();
    engine::game::effects::counters::remove_counter_with_replacement(
        runner.state_mut(),
        teferi,
        CounterType::Loyalty,
        2,
        &mut events,
    );
    engine::game::layers::evaluate_layers(runner.state_mut());

    assert_eq!(
        runner.state().objects[&teferi].loyalty,
        Some(2),
        "2 combat damage must remove 2 loyalty (4 → 2)"
    );
}
