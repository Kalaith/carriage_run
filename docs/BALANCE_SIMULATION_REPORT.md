# Balance Simulation Report

Date: 2026-08-05

## Harness

The deterministic harness lives in `src/state/tests/balance.rs`,
`balance_report.rs`, and `balance_expedition.rs`. It executes the real
`MissionRun` loop without rendering and injects boost, cruise, brake, or mixed
driver policies. The full report covers every mission route, difficulty,
chassis, and frame with two deterministic seeds and records success rate,
remaining health, cargo, special meter, deadline margin, enemy and hazard
encounters, reward, and failure penalty.

Expedition projections cover every leg, stake, modifier, and individual
starting relic. A seeded live-run sample additionally verifies survival and
payout reproducibility. Run the inspectable CSV report with:

```powershell
cargo test state::tests::balance_report -- --nocapture
```

## Stable corridors

These are deterministic invariants, not claims about human win rates:

- the same inputs produce byte-for-byte-equivalent simulation results;
- win counts and aggregate remaining health are strictly ordered by difficulty.
  The current 36-run fixed sample records wins `22 > 19 > 18` and remaining
  health totals Relaxed `30.977`, Standard `27.894`, Hard `24.342`;
- every timed route on Standard has a successful combination of frame,
  equipment/loadout, and boost-or-mixed policy for each chassis;
- mud preserves strictly more cargo under brake than cruise, and cruise more
  than continuous boost; and
- continuous boost is not universally best expected value.

On the seeded Muddy Road sample, boost/cruise/brake/mixed finished with cargo
ratios `0.956 / 0.962 / 0.968 / 0.979` and rewards
`352 / 352 / 477 / 429`. Brake encounters more threats because it remains on
the road longer, explaining its higher kill reward; mixed driving gives the
best cargo outcome. This is a policy comparison, not a recommendation to brake
through every mission.

## Campaign economy

First-clear projections include authored base reward, star pay, and a neutral
cargo allowance. Cumulative income at the rank milestones is:

| Outcome | Rank 2 (1 clear) | Rank 3 (4 clears) | Rank 4 (8 clears) |
| --- | ---: | ---: | ---: |
| 1-star | 173 | 812 | 1,849 |
| 2-star | 205 | 940 | 2,105 |
| 3-star | 237 | 1,068 | 2,361 |

Expected purchase space is therefore one early equipment upgrade at rank 2, a
Standard Wagon plus a rank-2 recruit by rank 3, and a Heavy Wagon plus a
rank-3 recruit by rank 4. These are options rather than mandatory paths; rank
advancement depends only on distinct mission clears.

## Expedition economy

With one modifier held constant for comparison, an eight-leg provisions path
ranges from `826` gold (no stake, Quiet Stretch, no Ledger) to `3,790` gold
(High Wager, Rich Haul, Merchant's Ledger). Failure salvage ranges from `413`
to `1,895`. Real seeded runs mix modifiers and events, so these are corridor
edges rather than expected payouts.

Paid stakes now block departure when unaffordable instead of silently falling
back. Permanent relics form a collection, but only two selected relics may
start together; this prevents the former six-relic stack while preserving
meaningful relic strength and player choice.

## Tuning decisions

- Mud loss scales with squared throttle, giving brake defensive value and boost
  a cargo cost without adding a new heat resource.
- Siege Supply retains larger, separated mega-waves, but uses a `0.60` combat
  pressure factor, five-to-ten-enemy bursts, and gentler momentum drain. This
  makes both routes viable with the Scout Cart when the Repair Kit and a
  suitable frame/policy are used.
- Timed loadouts calculate route demand against cruise capacity and visibly
  mark combinations that need active boost.
- No human win-rate target is asserted. Future playtest data should be compared
  with the inspectable matrix before adding such a corridor.
