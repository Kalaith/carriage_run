use super::super::*;

fn prisoner_run() -> MissionRun {
    let data = crate::data::GameData::load().unwrap();
    let campaign = CampaignState::new(&data.config, Some("muddy_road"));
    MissionRun::new(data.missions.get("prisoner_wagon").unwrap(), &campaign)
}

#[test]
fn breakout_attempts_are_telegraphed_and_counterable() {
    let mut run = prisoner_run();
    run.breakout_timer = 0.0;
    run.update_mission_pressure(0.1);
    assert_eq!(run.breakout_attempts, 1);
    assert!(run.breakout_progress > 0.0);

    run.throttle = 0.72;
    let before = run.breakout_progress;
    run.update_mission_pressure(0.5);
    assert!(
        run.breakout_progress < before,
        "braking should counter an attempt"
    );
}

#[test]
fn an_unchecked_breakout_marks_the_prisoner_escaped() {
    let mut run = prisoner_run();
    run.guards.clear();
    run.breakout_progress = 0.99;
    run.breakout_timer = 8.0;
    run.update_mission_pressure(1.0);
    assert_eq!(run.special_meter, 100.0);
}
