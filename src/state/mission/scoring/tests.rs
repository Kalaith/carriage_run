use super::*;
use crate::data::GameData;

#[test]
fn bonus_objective_adds_fifteen_percent_and_breakdown_reconciles() {
    let data = GameData::load().unwrap();
    let campaign = CampaignState::new(&data.config, Some("muddy_road"));
    let mut met_mission = data.missions.get("muddy_road").unwrap().clone();
    let mut missed_mission = met_mission.clone();
    met_mission.bonus.as_mut().unwrap().threshold = 0.5;
    missed_mission.bonus.as_mut().unwrap().threshold = 1.1;
    let run = MissionRun::new(&met_mission, &campaign);

    let met = run.make_report(&met_mission, true, "done");
    let missed = run.make_report(&missed_mission, true, "done");
    let expected_bonus = (run.base_reward as f32 * 0.15).round() as i64;

    assert_eq!(met.bonus_met, Some(true));
    assert_eq!(missed.bonus_met, Some(false));
    assert_eq!(met.reward - missed.reward, expected_bonus);
    assert_eq!(met.reward_breakdown.bonus_objective, expected_bonus);
    assert_eq!(met.reward, met.reward_breakdown.total());
}

#[test]
fn failed_mission_never_receives_bonus_gold() {
    let data = GameData::load().unwrap();
    let mission = data.missions.get("muddy_road").unwrap();
    let campaign = CampaignState::new(&data.config, Some("muddy_road"));
    let run = MissionRun::new(mission, &campaign);

    let report = run.make_report(mission, false, "failed");

    assert_eq!(report.bonus_met, Some(false));
    assert_eq!(report.reward_breakdown.bonus_objective, 0);
    assert_eq!(report.reward, report.reward_breakdown.total());
}
