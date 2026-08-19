use super::*;
use crate::data::MissionDef;
use crate::state::CampaignState;

fn test_run() -> MissionRun {
    let config = crate::data::GameConfig {
        game_name: "carriage_run".to_owned(),
        display_name: "Carriage Run".to_owned(),
        save_slot: "campaign".to_owned(),
        version: "0.1.0".to_owned(),
        toolkit_revision: String::new(),
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
        act: 1,
        biome: "test_biome".to_owned(),
        boss_id: None,
        side_mission: false,
        hazard_palette: Vec::new(),
        reward_note: String::new(),
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
fn spearman_braces_against_every_charger() {
    for charger in [EnemyKind::Wolf, EnemyKind::AlphaWolf] {
        assert!(charger.is_charger());
        assert!(melee_bonus(GuardKind::Spearman, 1, charger) > 1.0);
        assert!(melee_bonus(GuardKind::Spearman, 3, charger) > 1.0);
    }
    assert_eq!(melee_bonus(GuardKind::Spearman, 3, EnemyKind::Bandit), 1.0);
}

#[test]
fn mud_jostles_less_when_braking_and_more_when_boosting() {
    let cargo_after_mud = |throttle: f32| {
        let mut run = test_run();
        run.throttle = throttle;
        run.hazards
            .push(Hazard::new(HazardKind::Mud, run.carriage.pos));
        run.handle_hazard_collisions(1.0 / 60.0);
        run.carriage.cargo
    };

    let braking = cargo_after_mud(0.72);
    let cruising = cargo_after_mud(1.0);
    let boosting = cargo_after_mud(1.32);
    assert!(braking > cruising, "braking should preserve more cargo");
    assert!(cruising > boosting, "boosting should jostle more cargo");
}

#[test]
fn authored_hazards_have_distinct_collision_effects() {
    let mut rockslide = test_run();
    let health = rockslide.carriage.health;
    rockslide
        .hazards
        .push(Hazard::new(HazardKind::Rockslide, rockslide.carriage.pos));
    rockslide.handle_hazard_collisions(1.0 / 60.0);
    assert!(rockslide.carriage.health < health);

    let mut fog = test_run();
    fog.hazards
        .push(Hazard::new(HazardKind::CursedFog, fog.carriage.pos));
    fog.handle_hazard_collisions(1.0 / 60.0);
    assert!(fog.carriage.slow_timer > 0.0);

    let mut night = test_run();
    night
        .hazards
        .push(Hazard::new(HazardKind::NightStretch, night.carriage.pos));
    night.handle_hazard_collisions(1.0 / 60.0);
    assert!(night.carriage.night_timer > 0.0);
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

fn apply_test_hit(run: &mut MissionRun, kind: GuardKind, stars: u8, enemy_id: u32) {
    let target = run
        .enemies
        .iter()
        .find(|enemy| enemy.id == enemy_id)
        .unwrap()
        .pos;
    run.apply_guard_hit(PendingGuardHit {
        kind,
        stars,
        enemy_id,
        enemy_kind: run
            .enemies
            .iter()
            .find(|enemy| enemy.id == enemy_id)
            .unwrap()
            .kind,
        damage: 10.0,
        origin: run.carriage.pos,
        target,
    });
}

#[test]
fn three_star_cleave_and_piercing_shot_damage_a_second_target() {
    for kind in [GuardKind::Swordsman, GuardKind::Archer] {
        let mut run = test_run();
        let pos = run.carriage.pos;
        run.enemies.push(Enemy::new(1, EnemyKind::Bandit, pos, 1.0));
        run.enemies
            .push(Enemy::new(2, EnemyKind::Bandit, pos + vec2(20.0, 0.0), 1.0));
        let second_before = run.enemies[1].health;

        apply_test_hit(&mut run, kind, 3, 1);

        assert!(run.enemies[1].health < second_before, "{:?}", kind);
    }
}

#[test]
fn crossbow_bonus_and_pin_apply_to_armored_bandits() {
    let mut one_star = test_run();
    one_star.enemies.push(Enemy::new(
        1,
        EnemyKind::ArmoredBandit,
        one_star.carriage.pos,
        1.0,
    ));
    let mut three_star = one_star.clone();

    apply_test_hit(&mut one_star, GuardKind::CrossbowGuard, 1, 1);
    apply_test_hit(&mut three_star, GuardKind::CrossbowGuard, 3, 1);

    assert!(three_star.enemies[0].health < one_star.enemies[0].health);
    assert!(three_star.enemies[0].slow_timer > 0.0);
}

#[test]
fn mage_stars_add_splash_and_guard_healing() {
    let mut run = test_run();
    run.guards[0].health -= 20.0;
    let guard_before = run.guards[0].health;
    let pos = run.carriage.pos;
    run.enemies.push(Enemy::new(1, EnemyKind::Bandit, pos, 1.0));
    run.enemies
        .push(Enemy::new(2, EnemyKind::Bandit, pos + vec2(20.0, 0.0), 1.0));
    let second_before = run.enemies[1].health;

    apply_test_hit(&mut run, GuardKind::Mage, 3, 1);

    assert!(run.enemies[1].health < second_before);
    assert!(run.guards[0].health > guard_before);
}

#[test]
fn three_star_shield_wall_reduces_nearby_carriage_damage() {
    let mut plain = test_run();
    plain.guards.clear();
    let mut shielded = plain.clone();
    shielded.guards.push(Guard::new(
        9,
        GuardKind::ShieldGuard,
        shielded.carriage.pos,
        1,
        1,
        3,
        None,
    ));

    plain.damage_carriage(10.0, 0.0, "hit");
    shielded.damage_carriage(10.0, 0.0, "hit");

    assert!(shielded.damage_taken < plain.damage_taken);
}

#[test]
fn two_star_profiles_exercise_authored_stat_progression() {
    for kind in GuardKind::all() {
        let one = GuardProfile::new(kind, 1, 1, 1);
        let two = GuardProfile::new(kind, 1, 1, 2);
        assert!(two.attack > one.attack, "{:?} attack", kind);
        assert!(two.max_health > one.max_health, "{:?} health", kind);
        assert!(!kind.ability_summary(1).is_empty());
        assert!(!kind.ability_summary(2).is_empty());
        assert!(!kind.ability_summary(3).is_empty());
    }
    assert!(GuardKind::ShieldGuard.threat_bonus(2) > GuardKind::ShieldGuard.threat_bonus(1));
}
