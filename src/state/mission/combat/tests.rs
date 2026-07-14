use super::*;
use crate::data::MissionDef;
use crate::state::CampaignState;

fn test_run() -> MissionRun {
    let config = crate::data::GameConfig {
        game_name: "carriage_run".to_owned(),
        display_name: "Carriage Run".to_owned(),
        save_slot: "campaign".to_owned(),
        version: "0.1.0".to_owned(),
        starting_gold: 100,
    };
    let campaign = CampaignState::new(&config, Some("muddy_road"));
    let mission = MissionDef {
        id: "muddy_road".to_owned(),
        name: "The Muddy Road".to_owned(),
        order: 1,
        mission_type: "cargo_transfer".to_owned(),
        route: "Forest Road".to_owned(),
        cargo: "Basic Supplies".to_owned(),
        objective: "Reach the village.".to_owned(),
        bonus_objective: "Keep cargo safe.".to_owned(),
        intro_text: String::new(),
        bonus: None,
        outro_text: String::new(),
        unlock_level: 1,
        distance: 500.0,
        difficulty: 1.0,
        base_reward: 100,
        enemy_mix: vec!["wolf".to_owned()],
        hazard_mix: Vec::new(),
        route_choices: Vec::new(),
        prerequisite_missions: Vec::new(),
        unlock_any_missions: Vec::new(),
        time_limit: None,
    };
    MissionRun::new(&mission, &campaign)
}

#[test]
fn alpha_wolf_outclasses_a_common_wolf() {
    let wolf = Enemy::new(1, EnemyKind::Wolf, vec2(0.0, 0.0), 1.0);
    let alpha = Enemy::new(2, EnemyKind::AlphaWolf, vec2(0.0, 0.0), 1.0);
    assert!(alpha.max_health > wolf.max_health);
    assert!(alpha.damage > wolf.damage);
    assert!(alpha.speed > wolf.speed);
}

#[test]
fn live_enemies_are_hard_capped() {
    let mut run = test_run();
    // Far more spawn attempts than the cap; count must never exceed it.
    for _ in 0..(MAX_LIVE_ENEMIES * 4) {
        run.spawn_enemy();
    }
    assert_eq!(run.enemies.len(), MAX_LIVE_ENEMIES);
}

#[test]
fn roaming_melee_guard_auto_hits_nearby_enemy() {
    let mut run = test_run();
    let guard_pos = run
        .guards
        .iter()
        .find(|guard| guard.kind == GuardKind::Swordsman)
        .unwrap()
        .pos;
    run.enemies.push(Enemy::new(
        99,
        EnemyKind::Wolf,
        guard_pos + vec2(28.0, 0.0),
        1.0,
    ));
    let before = run.enemies[0].health;

    run.update_guard_orders(0.2);

    assert!(run.enemies[0].health < before);
    assert!(run.guards.iter().any(|guard| guard.attack_flash > 0.0));
}

#[test]
fn roaming_guard_advances_on_distant_enemy_within_leash() {
    let mut run = test_run();
    // An enemy inside the leash but well beyond weapon reach should be
    // chased, not ignored.
    let target = run.carriage.pos + vec2(0.0, -190.0);
    run.enemies
        .push(Enemy::new(77, EnemyKind::Wolf, target, 1.0));
    let guard_id = run
        .guards
        .iter()
        .find(|guard| guard.kind == GuardKind::Swordsman)
        .unwrap()
        .id;
    let before = run
        .guards
        .iter()
        .find(|guard| guard.id == guard_id)
        .unwrap()
        .pos
        .distance(target);

    run.update_guard_orders(0.2);

    let after = run
        .guards
        .iter()
        .find(|guard| guard.id == guard_id)
        .unwrap()
        .pos
        .distance(target);
    assert!(after < before, "roaming guard should close on the threat");
}

#[test]
fn spiked_hubs_wound_adjacent_enemies() {
    let mut run = test_run();
    run.hub_damage = 20.0;
    run.enemies
        .push(Enemy::new(42, EnemyKind::Wolf, run.carriage.pos, 1.0));
    let before = run.enemies[0].health;

    run.update_enemies(0.5);

    assert!(run.enemies[0].health < before);
}

#[test]
fn killing_fleeing_thief_recovers_cargo() {
    let mut run = test_run();
    run.guards.clear(); // isolate the carriage so the bandit targets it
    let full_cargo = run.carriage.cargo;
    run.enemies
        .push(Enemy::new(55, EnemyKind::Bandit, run.carriage.pos, 1.0));

    // Advance until the bandit steals and turns to flee.
    for _ in 0..40 {
        run.update_enemies(0.2);
        if run.enemies.iter().any(|enemy| enemy.retreating) {
            break;
        }
    }
    let thief = &run.enemies[0];
    assert!(thief.retreating, "bandit should flee after stealing");
    assert!(thief.carried_cargo > 0.0);
    assert!(
        run.carriage.cargo < full_cargo,
        "cargo should drop on theft"
    );

    // Cutting it down before it escapes returns the stolen cargo.
    run.enemies[0].health = 0.0;
    run.cleanup_entities();
    assert!((run.carriage.cargo - full_cargo).abs() < 0.001);
}
