use super::*;
use macroquad::prelude::vec2;

#[test]
fn boss_phases_change_at_authored_health_thresholds() {
    let mut boss = BossState::new("ash_colossus");
    assert_eq!(boss.phase, BossPhase::Approach);
    boss.update(3.0, vec2(0.0, 0.0));
    assert_eq!(boss.phase, BossPhase::First);
    boss.damage(120.0);
    assert_eq!(boss.phase, BossPhase::Second);
    boss.damage(130.0);
    assert_eq!(boss.phase, BossPhase::Enraged);
}

#[test]
fn boss_defeat_emits_victory_once() {
    let mut boss = BossState::new("road_warden");
    assert_eq!(boss.damage(999.0), Some(BossEvent::Victory));
    assert!(boss.is_defeated());
    assert_eq!(boss.damage(1.0), None);
}
