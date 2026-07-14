//! Live mission behaviour: pacing, special meters, frames, and assists.

use super::*;
use macroquad::math::{vec2, Rect};

fn idle_input() -> MissionInput {
    MissionInput {
        mouse: vec2(0.0, 0.0),
        pressed: false,
        down: false,
        released: false,
        repair_pressed: false,
        play_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
        steer_left: false,
        steer_right: false,
        boost: false,
        brake: false,
    }
}

#[test]
fn siege_run_sim_stays_bounded_and_terminates() {
    let data = GameData::load().unwrap();
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    session.campaign.difficulty_preset = DifficultyPreset::Hard;

    // Drop straight into the densest mission (bypassing unlock gates for the
    // test) and run the full sim headlessly under worst-case spawn pressure.
    let mission = data.missions.get("siege_supply").unwrap();
    session.mission = Some(MissionRun::new(mission, &session.campaign));
    session.screen = Screen::Playing;

    let mut ended = false;
    for _ in 0..8000 {
        if session
            .update_play(&data, 1.0 / 60.0, idle_input())
            .is_some()
        {
            ended = true;
            break;
        }
        // Runaway spawns would blow well past the hard cap (48).
        let live = session.mission.as_ref().unwrap().enemies.len();
        assert!(live < 64, "live enemy count unbounded: {live}");
    }
    assert!(ended, "mission did not terminate within the frame budget");
}

#[test]
fn carriage_frames_trade_stats_and_exclude_each_other() {
    let data = GameData::load().unwrap();
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    session.sync_chassis(&data);
    let mission = data.missions.get("muddy_road").unwrap();

    // Standard frame is the balanced baseline.
    assert_eq!(session.campaign.carriage_frame_id, "standard");
    let base_health = MissionRun::new(mission, &session.campaign)
        .carriage
        .max_health;

    // Reinforced trades speed for health.
    assert!(session.select_frame(&data, "reinforced"));
    assert!(!session.select_frame(&data, "reinforced")); // re-selecting is a no-op
    let reinforced = MissionRun::new(mission, &session.campaign);
    assert!(
        reinforced.carriage.max_health > base_health,
        "reinforced frame should add health"
    );

    // Switching to racing is exclusive: it drops health below baseline and the
    // reinforced bonus is gone (only one frame active at a time).
    assert!(session.select_frame(&data, "racing"));
    assert_eq!(session.campaign.carriage_frame_id, "racing");
    let racing = MissionRun::new(mission, &session.campaign);
    assert!(
        racing.carriage.max_health < base_health,
        "racing frame should cost health"
    );

    // Hauler boosts cargo capacity.
    assert!(session.select_frame(&data, "hauler"));
    let hauler = MissionRun::new(mission, &session.campaign);
    assert!(
        hauler.carriage.max_cargo
            > MissionRun::new(mission, &{
                let mut c = session.campaign.clone();
                c.carriage_frame_id = "standard".to_owned();
                c.frame_cargo_mult = 1.0;
                c
            })
            .carriage
            .max_cargo,
        "hauler frame should add cargo capacity"
    );
}

#[test]
fn princess_comfort_rewards_smooth_driving_over_swerving() {
    let data = GameData::load().unwrap();
    let mission = data.missions.get("princess_road").unwrap();
    let config = test_config();
    let campaign = CampaignState::new(&config, Some("muddy_road"));

    let input = |left: bool, right: bool| MissionInput {
        mouse: vec2(640.0, 300.0),
        steer_left: left,
        steer_right: right,
        ..idle_input()
    };

    // Glide a steady line: no steering input, hold centre for a couple of seconds.
    let mut smooth = MissionRun::new(mission, &campaign);
    smooth.special_meter = 50.0;
    for _ in 0..120 {
        smooth.handle_input(input(false, false));
        smooth.update(mission, 1.0 / 60.0);
    }

    // Yank the wheel side to side: a jerky, uncomfortable ride.
    let mut jerky = MissionRun::new(mission, &campaign);
    jerky.special_meter = 50.0;
    for i in 0..120 {
        let right = (i / 6) % 2 == 0;
        jerky.handle_input(input(!right, right));
        jerky.update(mission, 1.0 / 60.0);
    }

    assert!(
        smooth.special_meter > jerky.special_meter,
        "smooth ride ({}) should be comfier than a jerky one ({})",
        smooth.special_meter,
        jerky.special_meter
    );
    assert!(
        smooth.ride_smoothness_multiplier().unwrap() > jerky.ride_smoothness_multiplier().unwrap(),
        "smoothness multiplier should favour the clean line"
    );
}

#[test]
fn siege_run_fields_larger_waves_than_a_normal_route() {
    let data = GameData::load().unwrap();
    let config = test_config();
    let campaign = CampaignState::new(&config, Some("muddy_road"));

    // Same difficulty via the mission data, but the siege run's spawn bursts and
    // opening calm are scaled up for the mega-wave rhythm.
    let siege = MissionRun::new(data.missions.get("siege_supply").unwrap(), &campaign);
    let steady = MissionRun::new(data.missions.get("bandit_bend").unwrap(), &campaign);

    let siege_wave = siege.debug_wave_size();
    let steady_wave = steady.debug_wave_size();
    assert!(
        siege_wave >= steady_wave * 2,
        "siege mega-wave not larger: {siege_wave} vs {steady_wave}"
    );
    assert!(
        siege.debug_initial_lull() > steady.debug_initial_lull(),
        "siege should open with a longer calm"
    );
}

#[test]
fn monster_egg_hatches_into_a_brood_instead_of_instant_fail() {
    let data = GameData::load().unwrap();
    let mission = data.missions.get("monster_egg").unwrap();
    let config = test_config();
    let campaign = CampaignState::new(&config, Some("muddy_road"));

    let mut run = MissionRun::new(mission, &campaign);
    // Spend the egg's stability: the next tick should hatch, not fail.
    run.special_meter = 0.0;
    let enemies_before = run.enemies.len();

    let report = run.update(mission, 1.0 / 60.0);
    assert!(
        report.is_none(),
        "a hatching egg must not instantly end the mission"
    );
    assert!(
        run.enemies.len() > enemies_before,
        "the brood did not erupt"
    );
    // Ticking again does not re-hatch (fires once).
    let brood = run.enemies.len();
    run.update(mission, 1.0 / 60.0);
    assert!(
        run.enemies.len() <= brood + 1,
        "the egg hatched more than once"
    );
}

#[test]
fn assist_toggles_soften_the_run() {
    let data = GameData::load().unwrap();
    let mission = data.missions.get("muddy_road").unwrap();
    let config = test_config();
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));

    // Sturdy Carriage grants bonus max health.
    let base_health = MissionRun::new(mission, &campaign).carriage.max_health;
    campaign.sturdy_carriage = true;
    let sturdy_health = MissionRun::new(mission, &campaign).carriage.max_health;
    assert!(
        sturdy_health > base_health,
        "sturdy carriage not applied: {sturdy_health} !> {base_health}"
    );

    // Slower Waves lengthens the opening lull before the first wave.
    campaign.sturdy_carriage = false;
    let base_lull = MissionRun::new(mission, &campaign).wave_pace;
    campaign.slower_waves = true;
    let slow = MissionRun::new(mission, &campaign);
    assert!(slow.wave_pace > base_lull, "slower waves not applied");

    // Both are plain settings toggles.
    let mut session = GameSession::new(&config, Some("muddy_road"));
    assert!(session.toggle_setting("slower_waves"));
    assert!(session.campaign.slower_waves);
    assert!(session.toggle_setting("sturdy_carriage"));
    assert!(session.campaign.sturdy_carriage);
}

#[test]
fn generous_timers_extends_timed_missions() {
    let data = GameData::load().unwrap();
    let mission = data.missions.get("courier_deadline").unwrap();
    let config = test_config();
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));

    campaign.generous_timers = false;
    let base = MissionRun::new(mission, &campaign).time_limit.unwrap();
    campaign.generous_timers = true;
    let extended = MissionRun::new(mission, &campaign).time_limit.unwrap();

    assert!(extended > base, "timer not extended: {extended} !> {base}");
    assert!(
        (extended - base - 15.0).abs() < 0.01,
        "wrong bonus: {extended} vs {base}"
    );
}

#[test]
fn difficulty_preset_scales_mission_difficulty() {
    let config = test_config();
    let mission = test_mission("muddy_road", &[], &[], 1);
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));

    campaign.difficulty_preset = DifficultyPreset::Standard;
    let standard = MissionRun::new(&mission, &campaign).difficulty;
    campaign.difficulty_preset = DifficultyPreset::Relaxed;
    let relaxed = MissionRun::new(&mission, &campaign).difficulty;
    campaign.difficulty_preset = DifficultyPreset::Hard;
    let hard = MissionRun::new(&mission, &campaign).difficulty;

    assert!((standard - 1.0).abs() < 1e-3, "standard was {standard}");
    assert!(
        relaxed < standard,
        "relaxed {relaxed} !< standard {standard}"
    );
    assert!(hard > standard, "hard {hard} !> standard {standard}");
}
