# Balance and Systems Remediation Plan

Date: 2026-08-05

## Purpose

This plan resolves the correctness, interaction, communication, and balance
issues found in the fresh whole-game review. The review covered the campaign,
expedition mode, mission verbs, route choices, carriage progression, equipment,
guards, rewards, and the supported Windows/WebGL publishing path.

The current game builds successfully and all automated tests pass, but those
tests do not yet protect several important cross-system invariants. Work is
therefore ordered so correctness and player-facing truth are restored before
balance numbers are tuned.

## Success Criteria

The remediation is complete when:

- expedition relics never reduce a stat unless their data explicitly describes
  that trade-off;
- paid expedition event choices cannot be selected without enough run gold;
- campaign and guard unlock progression has an explicit source that is not an
  accidental side effect of buying Iron Plating;
- expedition branches contain only the route and modifier effects shown to the
  player, and seeded composition is deterministic from documented inputs;
- every bonus objective has a defined, visible gameplay reward;
- guard and hazard descriptions agree with the implemented mechanics;
- boost, brake, chassis, and frame choices each have defensible use cases rather
  than one generally dominant option;
- deterministic tests cover the corrected interactions; and
- `cargo test` and `./publish.ps1` pass after each implementation phase.

## Phase 1: Correctness Blockers

### 1. Normalize armor units and relic application

Problem: `MissionRun::armor_reduction` is used as flat damage reduction, while
relic data describes small fractional-looking additions. `apply_relic` clamps
the combined value to `0.9`, so applying any relic can reduce upgraded campaign
armor from `1.8` or more to `0.9`.

Resolution:

1. Choose one unit for carriage armor. Prefer flat damage reduction because the
   existing campaign equipment and `damage_carriage` already use that model.
2. Rename relic data and Rust fields from `armor_add` to a unit-explicit name
   such as `flat_armor_add`.
3. Retune Iron Barding, War Banner, and Ghost Wheels in the chosen unit.
4. Remove the fractional clamp from `MissionRun::apply_relic`. If a cap is still
   required, define it in the same flat unit and apply it after all modifiers.
5. Keep Ghost Wheels' armor penalty explicit and ensure all zero-armor relics
   leave armor exactly unchanged.

Acceptance tests:

- applying Merchant's Ledger or Greased Axles preserves campaign armor;
- Iron Barding increases final damage reduction;
- War Banner increases final damage reduction by its authored amount;
- Ghost Wheels reduces armor only by its authored penalty; and
- carriage damage remains bounded and non-negative at maximum armor.

### 2. Enforce expedition event affordability

Problem: event options with negative gold are always enabled. Resolution clamps
run gold at zero, allowing the player to receive paid benefits without paying
the full cost.

Resolution:

1. Add a shared `can_choose_event_option` check on `Journey` or `GameSession`.
2. Reject any option whose negative gold delta exceeds `banked_gold`.
3. Use the same check to disable the choice in `ui/journey.rs`.
4. Preserve free alternatives so every event can always be resolved.
5. Decide explicitly whether entry-stake gold and campaign gold are ever valid
   event currencies. The recommended rule is that events spend run-banked gold
   only, matching the current display and risk loop.

Acceptance tests:

- a zero-gold run cannot pay the toll or buy a shrine relic;
- an affordable event deducts its exact cost;
- a rejected option does not clear the pending event or apply benefits; and
- every shipped event retains at least one selectable option at zero gold.

## Phase 2: Progression and Mode Boundaries

### 3. Separate campaign rank from Iron Plating

Problem: `carriage_level` simultaneously means campaign rank, Iron Plating
level, legacy chassis progression, mission unlock gate, and guard unlock gate.
Buying a real chassis does not advance content, while buying armor does.

Resolution:

1. Introduce an explicit persisted progression field, such as
   `campaign_rank`, with a documented advancement rule.
2. Rename or migrate `carriage_level` to `armor_level` for equipment strength.
3. Gate missions and guard availability on `campaign_rank`, not armor.
4. Choose a rank rule that rewards campaign progress. Recommended milestones:
   rank 1 at campaign start, then ranks 2, 3, and 4 after completing defined
   mission-order or act thresholds.
5. Migrate existing saves conservatively. Derive the initial rank from completed
   mission records, then use legacy `carriage_level` only as a fallback so no
   established save loses access.
6. Update all UI labels and descriptions so rank, chassis, armor, and equipment
   level are distinct concepts.

Acceptance tests:

- buying Iron Plating never unlocks a mission or guard by itself;
- completing the documented milestone advances campaign rank;
- buying or selecting a chassis changes slots and chassis stats only;
- legacy saves retain every mission and guard they previously had access to;
  and
- the full mission graph remains reachable without a mandatory armor build.

### 4. Isolate expedition legs from campaign route selections

Problem: expedition legs call `MissionRun::new`, which silently applies the
campaign profile's stored route choice for that mission. The expedition UI only
shows its generated base mission and leg modifier.

Resolution:

1. Add an explicit mission-run construction context for campaign versus
   expedition.
2. Apply `CampaignState::selected_route_choice` only to campaign runs.
3. Give expedition legs a neutral base route or include an expedition route
   choice directly in `LegOption` and display it before selection.
4. Document which profile inputs intentionally affect seeded runs. Loadout and
   permanent unlocks may affect power, but the generated leg sequence should be
   identical for the same seed.

Acceptance tests:

- changing a campaign route selection does not change an expedition leg's
  distance, deadline, enemy mix, hazard mix, or reward;
- identical seeds generate identical leg and event sequences across otherwise
  equivalent sessions; and
- the displayed leg description accounts for every modifier applied at start.

## Phase 3: Rewards and Player-Facing Truth

### 5. Give bonus objectives an explicit reward

Problem: bonus objectives produce a Met/Missed badge but do not change score,
stars, gold, unlocks, or records. Some happen to correlate with other scoring
inputs, but the objective itself has no bonus.

Resolution:

1. Choose one consistent reward. Recommended: a mission-defined bonus-gold
   amount derived from base reward, such as 15%, with a clear results row.
2. Evaluate the objective before final reward calculation.
3. Record bonus completion separately if future achievements or route mastery
   will use it.
4. Ensure the threat-count bonus and other non-star criteria receive the same
   explicit treatment as cargo, health, time, and special-meter objectives.

Acceptance tests:

- otherwise identical successful reports differ by the documented bonus amount
  when the criterion is met;
- failed missions never receive the bonus;
- the results breakdown reconciles exactly to awarded gold; and
- Bandit Bend's threat objective provides a real benefit beyond per-kill gold.

### 6. Reconcile descriptions with mechanics

Resolve each mismatch by either implementing the promised behavior or narrowing
the text. Prefer mechanics when they create useful counterplay; prefer text-only
corrections when the promised behavior would add complexity without a decision.

Required decisions:

- Spearman: extend the brace bonus to Alpha Wolves and any future enemy tagged as
  a charger, preferably through an enemy capability rather than a hard-coded ID.
- Mage: either introduce enemy armor and genuine armor bypass, or replace the
  one-star text with the actual magic-bolt behavior. Do not add a new armor
  system solely to preserve one sentence.
- Armored Bandit: describe high durability accurately if armor remains a fiction
  represented by health; keep the Crossbow's authored damage bonus visible.
- Mud: either scale cargo jostling with throttle and reduce it while braking, or
  remove the claim that speed changes cargo loss. Implementing throttle-sensitive
  loss is preferred because it gives brake a concrete defensive purpose.

Acceptance tests:

- Spearman bonuses cover every enemy advertised as a charging counter;
- each star ability summary maps to an exercised code path; and
- mud collision tests compare braking, cruising, and boosting outcomes if
  throttle-sensitive loss is implemented.

## Phase 4: Balance Tuning

### 7. Make boost and brake situational choices

Problem: boost provides unlimited `1.32x` progress. Because enemy waves and
hazards spawn against elapsed time, continuous boost also reduces total exposure
and improves timed scoring. Its softer steering response is the only general
cost. Brake extends exposure and currently has little compensating value.

Resolution approach:

1. Build the deterministic simulation described in Phase 5 before changing
   numbers.
2. Measure continuous boost, cruise, and selective brake strategies across all
   missions, chassis, frames, and difficulties.
3. Add a trade-off only if the data confirms dominance. Candidate designs, in
   preferred order:
   - boost increases collision and mud cargo loss while brake reduces it;
   - boost accumulates heat or strain and must recover at cruise;
   - boost increases wave alert or enemy catch-up pressure; or
   - boost consumes a small route-limited resource.
4. Preserve the Princess mission's smooth-driving identity rather than making
   its steering penalty the universal solution.
5. Retune timed missions after the final throttle model, particularly both
   Siege Supply routes and Heavy Wagon combinations.

Acceptance corridors:

- continuous boost is not the best expected-value strategy on every non-Princess
  mission;
- selective braking improves at least one meaningful survival or cargo outcome;
- every timed route is feasible on Standard difficulty with every chassis when
  using a suitable available frame/equipment/loadout, with the UI warning about
  combinations that require active boosting; and
- Relaxed, Standard, and Hard win rates remain strictly ordered.

### 8. Audit campaign and expedition economies

The static review found 2,435 gold in listed campaign base rewards against 3,375
gold in upgrade purchases from new-game levels, before chassis, recruits, and
guard-star costs. This supports meaningful scarcity on a single clear, but the
actual economy also includes stars, cargo, special meters, kills, bonuses,
failures, replays, stakes, relic multipliers, repairs, and event costs.

Resolution:

1. Simulate first-clear campaign income at one-, two-, and three-star outcomes.
2. Define expected purchase milestones for ranks 2 through 4.
3. Verify that no upgrade path is mandatory after campaign rank is separated
   from armor.
4. Measure expedition payout distributions for cash-out and failure at every
   leg, with all stake tiers.
5. Review permanent starting relic stacking. If six simultaneous starting
   relics trivialize later runs, add a starting-relic slot limit or a pre-run
   selection rather than weakening each relic into insignificance.
6. Ensure unaffordable stakes visibly fall back or, preferably, block starting
   until the player selects an affordable tier.

Acceptance corridors should be recorded beside the simulator once observed
playtest data exists. Do not invent final win-rate targets from code inspection
alone.

## Phase 5: Validation Infrastructure

### 9. Add a deterministic balance simulation harness

Create a headless harness over `MissionRun` that can execute reproducible driver
and loadout policies without rendering. Keep it inside the crate's unit-test
module structure so it can access binary-crate internals, following
`CODE_STANDARDS.md`.

Minimum outputs:

- success rate by mission, route, difficulty, chassis, and frame;
- mean remaining health, cargo, special meter, and deadline margin;
- enemy and hazard counts encountered;
- reward, failure penalty, and campaign purchase timing;
- expedition survival and payout by leg, stake, modifier, and starting relic;
  and
- comparisons for boost, cruise, brake, and mixed driving policies.

Use corridor assertions only for invariants that should remain stable, including
difficulty monotonicity, seeded reproducibility, non-dominant throttle policies,
and minimum viability. Keep broader tuning reports inspectable without making
every numerical adjustment break the build.

### 10. Validation sequence for every phase

For each implementation phase:

1. Add a regression test that fails against the old behavior.
2. Implement the smallest cohesive correction.
3. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
   `cargo test` during development.
4. Run `./publish.ps1` with no parameters as the final project validation path.
5. Capture affected UI scenes when wording, reward breakdowns, disabled choices,
   or progression labels change.
6. Update this plan by marking completed sections and recording any deliberate
   design decision that differs from the recommendation.

## Recommended Delivery Order

Keep the work in focused commits so regressions and balance changes can be
reviewed independently:

1. armor-unit correction and relic regression tests;
2. expedition event affordability and UI state;
3. campaign-rank migration and unlock rewiring;
4. expedition run-construction boundary;
5. bonus reward calculation and results breakdown;
6. description/mechanic reconciliation;
7. deterministic balance harness;
8. throttle and timed-route tuning based on harness results; and
9. campaign/expedition economy tuning and final UI verification.

## Out of Scope

This plan does not add new campaign acts, bosses, art, audio, localization,
controller support, or platform features. Those remain in `TODO.md`. New content
would obscure whether the existing system interactions are corrected and should
resume only after the correctness phases and initial balance harness are in
place.
