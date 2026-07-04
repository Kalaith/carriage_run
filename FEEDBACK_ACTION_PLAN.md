# Feedback Action Plan — 2026-07-05

Source: [feedback.md](feedback.md). Each feedback item mapped to concrete changes with file references.

---

## P0 — Game Balance (feedback #1, #4: "map 2 ramps up too fast")

The numbers confirm the feedback. Going from mission 1 to mission 2:

| | Muddy Road | Bandit Bend | Change |
|---|---|---|---|
| Difficulty | 1.0 | 1.22 | +22% spawn rate, +HP/dmg scaling |
| Distance | 940 | 1160 | +23% longer exposure |
| Enemy mix | wolf only | wolf + 2× bandit + archer | first ranged enemy, 4 threat types |
| Reward before it | — | ~100 gold + 125 starting | buys only 2–3 level-1 upgrades |

Difficulty divides the spawn interval (`flow.rs:162-171`: `rng(1.4,2.8) / diff`), so 1.22 compounds with the longer distance and harder mix — three ramps at once.

### Actions

1. **Flatten the early curve** in [missions.json](assets/data/missions.json):
   - `bandit_bend`: difficulty 1.22 → **1.08**, distance 1160 → **1040**, enemy mix `["wolf", "bandit", "bandit"]` (drop one bandit, delay `bandit_archer` to route-choice `smugglers_cut` only).
   - `courier_deadline`: difficulty 1.16 → **1.10**.
   - Re-grade the rest so difficulty rises with `order` (currently order 4 = 1.48 but order 5 = 1.38): target roughly `1.0, 1.08, 1.10, 1.22, 1.30, 1.34, 1.42, 1.48, 1.52, 1.56, 1.60, 1.80`.
2. **Grace period at route start** in [flow.rs](src/state/mission/flow.rs): scale spawn rate by progress — e.g. multiply the spawn interval by `1.6` while `progress_ratio < 0.15`, tapering to `1.0` by 25%. Lets players settle guards before pressure peaks.
3. **Economy**: raise `muddy_road` base_reward 100 → **120** and/or reduce `guard_training` base_cost 55 → **45**, so a player entering map 2 can afford ~3 upgrades. Verify with a playthrough that 2 clears of map 1 fund a comfortable map 2 attempt.
4. **Regression check**: add a unit test in [tests.rs](src/state/tests.rs) asserting mission difficulty is non-decreasing by `order` (allowing branch-track exceptions if intended).

---

## P1 — Speed Feedback (feedback: "no clear sign what speed the player is going at")

No speed is displayed anywhere ([gameplay_hud.rs](src/ui/gameplay_hud.rs) shows only health/cargo/progress). Speed exists as `scroll_speed()` and `speed_factor()` in [mission.rs:288](src/state/mission.rs:288).

### Actions

1. **HUD speed indicator** in gameplay_hud.rs: a small gauge/label showing current speed derived from `scroll_speed()`, normalized so base = "10 mph"-style readable number or a 5-segment bar.
2. **State color**: normal (white), mud-slowed (amber, while `slow_timer > 0`), wheel-boosted above base (green). This makes mud hits and the Reinforced Wheels upgrade legible.

---

## P1 — Movement Feel (feedback: "feels less like the ground is moving than the caravan")

The carriage is fixed at `CARRIAGE_Y = 506`; only road lane markers scroll (`road_scroll % 96`, [gameplay.rs:46-144](src/ui/gameplay.rs)). The background is a static green fill, so there's nothing off-road to sell motion.

### Actions

1. **Scrolling roadside props**: spawn trees/rocks/bushes/fence posts on the grass margins that move down at `scroll_speed()` and recycle off-screen. This is the single biggest fix — motion needs reference objects.
2. **Parallax**: a second, sparser prop layer at ~0.6× scroll speed for depth.
3. **Wheel dust/mud particles** behind the carriage, emission rate tied to `speed_factor()` — doubles as speed feedback.
4. **Subtle carriage bob** (1–2 px vertical sine on the sprite only, not the hitbox) to sell travel over ground.

---

## P1 — Mission Map Declutter (feedback: "too much information on the initial screen; hide maps until close to unlocking")

[mission_map.rs:18-45](src/ui/mission_map.rs) draws all 12 missions in a grid regardless of unlock state; locked ones just get a dark overlay.

### Actions

1. **Visibility tiers** using `campaign.is_mission_unlocked()` ([campaign.rs:6-18](src/state/campaign.rs)):
   - **Shown normally**: unlocked or completed.
   - **Teaser card** ("???" + unlock hint only): missions exactly one step away (all-but-one prerequisite complete, or carriage level within 1 of `unlock_level`).
   - **Hidden entirely**: everything else.
2. **Add `is_mission_near_unlock()`** helper to campaign.rs with unit tests.
3. **Slim the detail panel** ([mission_map.rs:206-242](src/ui/mission_map.rs)): keep name/type/objective/reward/route choices visible; fold enemy mix, hazard mix, distance/difficulty behind a "Scout Report" expander so the default view is calmer.

---

## P2 — Guard Free-Roam Stance (feedback: "free roam option for the guards might be good, and direct when needed")

Guards have `Escort / Move / Hold / Attack` orders ([entities.rs:388-393](src/state/entities.rs)) driven by drag-drop ([flow.rs:95-130](src/state/mission/flow.rs)). Nothing self-directs beyond auto-attack in range.

### Actions

1. **New `GuardOrder::Roam` variant**: guard automatically intercepts the nearest enemy within a leash radius (~220 px) of the carriage, returns to escort position when no targets. Implement target selection in [combat.rs](src/state/mission/combat.rs) alongside the existing Escort auto-attack.
2. **Direct orders still override**: any drag order (Move/Attack) takes precedence; guard returns to Roam (not Escort) when the ordered task completes, if Roam was its stance.
3. **UI toggle**: small stance icon per guard (or click-guard-without-drag cycles Escort ↔ Roam), with a distinct ring color so stance is readable at a glance.
4. Consider making **Roam the default** for melee guards after playtesting — it matches how the reviewer expected guards to behave.

---

## P2 — Carriage Equipment Milestone (feedback #6: "equipment for carriage, carriage slot and size, maybe buy new caravan with more slots")

Today: 4 equipment types, slot count derived from `carriage_level` (2/3/3/4, [equipment.rs:66-74](src/state/equipment.rs)), no carriage purchase mechanic.

### Actions (larger feature — sequence after P0/P1 land)

1. **Carriage chassis as a purchasable**: add `CarriageChassis` (e.g. Scout Cart — 2 slots, +speed; Standard Wagon — 3 slots; Heavy Wagon — 4 slots, +HP, −speed) with gold costs, stored on campaign state. Slot count moves from `carriage_level` to chassis; `carriage_armor` upgrade keeps its HP/armor role.
2. **Data-driven**: new `assets/data/carriages.json`, loaded in [data.rs](src/data.rs), mirroring how upgrades.json works.
3. **Shop UI**: extend [management.rs](src/ui/management.rs) / [carriage.rs](src/ui/carriage.rs) with a chassis tab; update `CarriageVisual` to render per-chassis silhouettes.
4. **2–3 new equipment items** to make slot choice meaningful at 4 slots (e.g. Lantern — slows night/undead spawn pressure; Spiked Hubs — contact damage; Horn — brief enemy scatter, on cooldown).
5. Save-compat: default existing saves to Standard Wagon.

---

## P3 — Light Story Framing (feedback #2: "game doesn't have a story")

Reviewer noted the absence rather than demanding a campaign plot. Cheap, high-flavor fix:

1. Add a 1–2 sentence `intro_text` per mission in missions.json, shown on the mission detail panel / loadout screen ([loadout.rs](src/ui/loadout.rs)) — a courier-log vignette connecting the maps into one journey.
2. Optional: a short epilogue line on the mission-complete screen ([scoring.rs](src/state/mission/scoring.rs) results path).

---

## Suggested order of work

1. **P0 balance** (JSON tuning + grace period + economy) — small diffs, biggest complaint.
2. **P1 speed HUD + movement feel** — visual polish, no systems risk.
3. **P1 mission map declutter** — contained to mission_map.rs + one campaign helper.
4. **P2 guard Roam stance** — new order variant + UI.
5. **P2 carriage chassis system** — the reviewer's stated "next milestone", largest feature.
6. **P3 story flavor text** — anytime, content-only.

No action needed for items 3, 5, 7 (positive feedback: reverse-tower-defense feel, danger-vs-exit tension, streamer appeal) — preserve these while changing balance.
