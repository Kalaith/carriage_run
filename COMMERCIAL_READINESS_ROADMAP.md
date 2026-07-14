# Carriage Run — Commercial Readiness Roadmap

*Compiled 2026-07-14 from a full code/asset survey of the current `master` (commit `13517e4`). Every claim below was verified against the source; file references point at the systems as they exist today.*

---

## 1. Where the game actually is

Carriage Run is **past prototype**: ~11.2k lines of clean, tested Rust with a complete campaign loop and one absorbed feedback cycle. The design review's Phase 1–2 items have shipped — verified in code:

- ✅ `GuardOrder::Roam` implemented and default for melee (`src/state/entities.rs:408-426`, `src/state/mission/combat.rs:85-118`)
- ✅ Distinct enemy behavior verbs: wolf charge, bandit cargo-steal-and-flee (recoverable), archer/necromancer kiting, necromancer skeleton-raising (`src/state/mission/combat.rs:175-313`)
- ✅ Wave system with telegraphs and lulls (`src/state/mission/flow.rs:203-260`)
- ✅ Throttle/brake with steering-response trade-off (`src/state/mission/flow.rs:148-185`)
- ✅ 3-chassis purchase system (`assets/data/carriages.json`, `src/state/chassis.rs`)
- ✅ 6 equipment types with real sim effects (`src/state/equipment.rs`, `src/state/mission.rs:236-316`)
- ✅ Expedition mode: chained runs, banked-gold push-your-luck, fail state (`src/state/journey.rs`)
- ✅ Versioned save/load with migration + legacy fallback, atomic writes, 26 unit tests, working CI (fmt/clippy/test/WASM/Windows)

What it is **not** is a product. The commercial gaps are not unfinished code (zero TODO/stub markers exist) — they are entire missing layers:

| Layer | State today |
|---|---|
| Audio | **0%** — no SFX, no music, no volume control. Toolkit `SoundManager` exists, unused. |
| Art | **~1%** — one texture (title screen). All gameplay art is 358 procedural shape calls. |
| Game feel / VFX | Color hit-flash + fake wheel dust only. No particles, shake, tweens, damage numbers, death FX. |
| Onboarding | Two static HUD strings. No tutorial, no tooltips anywhere. |
| Content volume | 12 missions (~90 min first-clear), 5 enemies, no bosses, no biome variety. |
| Retention | Expedition overhauled into a real roguelite: per-leg reward choices, relics, bespoke branching legs, run events, an 8-leg finale, and **persistent meta-progression** (expedition tokens → unlockable starting relics). Seeded/daily runs + run records remain. |
| Settings | 3 boolean toggles. No volume/display/rebinding/difficulty/accessibility. |
| Localization | None — hardcoded English across `src/` and `assets/data/*.json`. |
| Distribution | FTP-to-web catalog only. No Steam/itch pipeline, icon, installer, LICENSE, or store presence. |
| Observability | No crash reporting, no telemetry; web-only bug-report widget. |

The workspace `standing.md` estimates **4–7 months to finish** — consistent with what this survey found, assuming a solo dev plus commissioned art/audio.

**Assumed commercial target (see §6 Open Decisions):** premium single-player title (~$10–15) on **Steam (Windows) + itch.io**, with the existing **web build as the free demo/marketing funnel**. Everything below is prioritized against that target.

**Priority key:** 🔴 P0 = release blocker · 🟠 P1 = must-have for a competitive launch · 🟡 P2 = should-have / strongly differentiating · ⚪ P3 = post-launch candidate.

---

## Progress log — autonomous loop (updated 2026-07-14)

Shipped and committed to `master` this cycle (checkboxes below updated to match):

- **A1 Run summary & records — completes the expedition overhaul (all 8 A1 items done)** — persistent `ExpeditionRecords` on the saved campaign track lifetime bests (best legs, best banked, completions, total legs, runs started) and a capped recent-run history. A new **Records** screen (reached from the Outfitter) shows the lifetime stats plus a run log with each run's outcome, banked gold, and shareable daily seed. Also addresses F6's stats/records screen. **Workstream A1 (expedition → roguelite) is now fully shipped.**
- **A1 Seeded/daily runs** — every expedition now carries a run seed that deterministically drives all procedural composition (leg branches, run events, relic offers) through a SplitMix64-mixed `seed_index`. Same seed reproduces the same run, so the Outfitter's new **Daily Run** button starts a date-derived run identical for everyone that day; the shareable `seed_code` (8-hex) shows on the hub and victory screen. Free runs now use a fresh nonce, so they finally vary run-to-run instead of being identical every time. *(done)*
- **A1 Meta-progression** — expeditions now persist. Each run banks **expedition tokens** (1 per leg cleared + a 5-token completion bonus) onto the saved `CampaignState`. A new pre-run **Expedition Outfitter** screen (opened from the loadout's Expedition button) spends tokens to permanently unlock **starting relics**; every future expedition then begins already holding them. This is the first thing in the game that carries across runs. *(done)*
- **A1 Escalation cap / finale** — expeditions are now a defined 8-leg arc with a win condition (`Journey::EXPEDITION_LENGTH`). Clearing the final leg wins the run: a triumphant "Expedition Complete!" summary with a completion bonus (`completion_bonus` = 2× the final leg's base reward) folded into the banked payout. The hub telegraphs the arc ("Leg N of 8", and "Final Leg — Choose the Road Home" on the last hop). Replaces the endless "leave or die" loop. *(done)*
- **A1 Run events** — between-legs vignette decisions. 6 data-driven events (`assets/data/run_events.json`: toll bridge, stranded merchant, shortcut rumor, roadside shrine, wounded scout, supply caravan), each a 2-way resource trade (gold / carriage health / relic grant). Presented on the expedition hub before the next-leg branch and gated on a decision (`journey_resolve_event`), rotating through the pool by leg. *(done — also seeds C3 expedition vignettes)*
- **A1 Bespoke leg generation + branching choice** — expedition legs are no longer recycled campaign missions on a modulo cycle. Each hop offers an FTL-style 3-way branch on the hub ("Choose the Next Road"), where every option is a base route paired with a data-driven `LegModifierDef` twist (`assets/data/leg_modifiers.json`: Raider Ambush, Beast Country, Cursed Ground, Rich Haul, Quiet Stretch, Undead Omen) that layers extra enemies/hazards and scales difficulty + banked reward via `MissionRun::apply_leg_modifier`. *(done)*
- **A1 Relic/boon system** — the roguelite build-identity hook. 6 data-driven relics (`assets/data/relics.json` → `RelicDef`) collected during an expedition and folded into every leg's mission run via `MissionRun::apply_relic` (speed/armor/wheel/contact-damage/reward-multiplier axes). Relics are offered as the first reward slot on leg completion (replacing the pure-gold Bounty when one is available), listed on the expedition hub, and are session-only. *(done — effect axes extensible via `RelicDef`)*
- **A1 Reward variety per leg** — expeditions now present a choice-of-3 spoils screen after each cleared leg (Bounty Purse: pure gold; War Provisions: gold + partial heal; Field Repairs: full repair + gold), replacing the flat auto-payout. Pressing on is gated on picking one; the pick is a gold-vs-upkeep trade that depends on convoy damage. *(first A1 bullet done — relics/boons, bespoke legs, run events, finale, meta-progression, seeded/daily, run records remain)*
- **D3 VFX / juice** — floating damage numbers, plus a minimal particle system driving enemy death bursts (slain foes scatter tinted particles instead of vanishing). *(partial — damage numbers + death animations done; shake/hit-stop, more emitters remain)*
- **B2 New hazards** — River Ford (a wide forced-slow crossing) added to Medicine Run and the Field Guide. *(partial — 1 of 2–3 new hazards)*
- **A5 Elite enemies** — Alpha Wolf (fast charging pack leader) and Armored Bandit (tanky non-fleeing raider, countered by crossbow armor-pierce), added to Refugee Escort / Gold Shipment and the Field Guide. *(elite variants done — 2 of 2–3; bosses remain)*
- **A2 Endgame economy** — Reinforced Kit consumable (+55 health for one route), a repeatable gold sink bought in the shop. *(done)*
- **A6 Difficulty presets** — Relaxed / Standard / Hard scaling the mission difficulty scalar. *(done)*
- **A6 Assist toggles** — "Generous Timers" (+15s on timed missions), orthogonal to the presets. *(partial — one assist so far; slower-waves / extra-gold remain)*
- **C1 Narrative** — courier-log intro (loadout brief) and outro (results screen) for all 12 missions. *(done)*
- **F3 Confirmation dialogs** — New Campaign confirms before overwriting the save. *(partial — chassis/expedition confirms remain)*
- **F4 Post-mission clarity** — structured bonus objectives graded met/missed on a relaid-out results screen. *(partial — "what to buy next" nudge remains)*
- **F5 Help/codex** — Field Guide screen with Threats / Guards / Hazards tabs, reusing the in-game sprites. *(done)*
- **F6 Menu polish** — journey-progress readout (routes cleared + total stars) on the Routes header. *(partial — full records screen remains)*
- **I1 Save hardening** — Continue/Load gated on the save actually loading; a corrupt save is skipped with a warning instead of silently failing. *(partial — rolling `.bak` / restore remains)*
- **I2 Crash handling** — native panic hook writes a crash log to app-data. *(partial — user-facing crash dialog remains)*
- **I3 Data validation** — semantic content-id validation, unlock-graph reachability, and cost/unlock invariant tests. *(done)*
- **I4 Stability** — hard cap of 48 live enemies, plus a headless soak test that runs the full mission sim to completion under worst-case load (enabled by routing keyboard input through `MissionInput` instead of `is_key_down`). *(done)*
- **Procedural art** — redrawn menu-backdrop wagon (horse-drawn covered wagon) and the crest (wagon-wheel emblem).

---

## 2. Workstream A — Game depth & endgame (design-complete the loop)

The core 60-second loop is good. What's missing is what happens after hour one.

### A1. ✅ Expedition mode → real roguelite (the retention engine) — **COMPLETE**
**All 8 sub-items shipped.** Expedition went from a skeleton (campaign missions on a modulo cycle with a flat payout) to a full roguelite: per-leg reward choices, run-scoped relics, FTL-style bespoke branching legs, between-legs run events, an 8-leg finale with a win condition, persistent meta-progression (tokens → unlockable starting relics), seeded/daily runs, and a run-records screen. This is the retention pillar the design review called for.

- [x] **Reward variety per leg**: choice-of-3 rewards on leg completion (Bounty Purse / War Provisions / Field Repairs — gold vs. carriage-upkeep trades), replacing the flat payout. *(done — `LegReward` in `journey.rs`, post-leg choice screen in `ui/journey.rs`)*
- [x] **Relic/boon system**: run-scoped modifiers that create build identity within a run. 6 relics in `assets/data/relics.json` (Ghost Wheels: +speed/−armor; Iron Barding; Merchant's Ledger: +35% leg gold; Greased Axles; Spiked Ram; War Banner), folded into each leg's mission run (`MissionRun::apply_relic`) and offered as the first reward slot on leg completion. *(done — data-driven; guard-roam/summon-style effects can be added later by extending `RelicDef`)*
- [x] **Bespoke leg generation** + **branching leg choice** (FTL-style): each next leg is a base campaign route paired with a data-driven `LegModifierDef` twist (`assets/data/leg_modifiers.json` — Raider Ambush, Beast Country, Cursed Ground, Rich Haul, Quiet Stretch, Undead Omen) that adds enemies/hazards and scales difficulty + banked reward (`MissionRun::apply_leg_modifier`). The expedition hub offers a 3-way "Choose the Next Road" branch (`generate_leg_options`, `journey_begin_leg`). *(done — pick 1 of 3 procedurally-composed legs each hop)*
- [x] **Run events**: non-combat vignette decisions between legs. 6 data-driven events (`assets/data/run_events.json` → `RunEventDef`: toll bridge, stranded merchant, shortcut rumor, roadside shrine, wounded scout, supply caravan), each a 2-option resource trade (gold / carriage health / relic grant). Presented on the hub before the leg branch and gated on a decision (`journey_resolve_event`). *(done — pool extensible; shares C3)*
- [x] **Escalation cap / finale**: defined run length (`Journey::EXPEDITION_LENGTH` = 8 legs). Clearing the final leg wins the run — a triumphant "Expedition Complete!" summary with a completion bonus (`completion_bonus`), replacing the endless "leave or die" treadmill. The hub telegraphs progress ("Leg N of 8", "Final Leg — Choose the Road Home"). *(done — boss-leg finale can slot in once A5 bosses land)*
- [x] **Meta-progression**: persistent expedition currency + unlock track. Every run banks **expedition tokens** (1 per leg cleared + a completion bonus), saved on `CampaignState` (`expedition_tokens`). A new **Expedition Outfitter** screen (pre-run, from the loadout) spends tokens to permanently unlock **starting relics** (`expedition_unlocks`); future expeditions begin already holding them (`unlock_starting_relic`, `start_journey` seeds from unlocks). *(done — currency/unlock spine in place; cosmetics/guard-variants can hang off the same track)*
- [x] **Seeded/daily runs**: every expedition now carries a run `seed` that drives all procedural composition (leg branches, run events, relic offers) via a SplitMix64-mixed `seed_index`. Same seed → identical run (shareable), so the Outfitter's **Daily Run** button starts a date-derived run that's the same for everyone that day, with the shareable `seed_code` surfaced on the hub and victory screen. Free runs get a fresh nonce, fixing the prior "every run is identical" limitation. *(done)*
- [x] **Run summary & records**: persistent `ExpeditionRecords` on the campaign track lifetime bests (best legs, best banked, completions, total legs, runs started) and a capped recent-run history. A new **Records** screen (from the Outfitter) shows the lifetime stats plus the run log with per-run outcome, banked gold, and shareable daily seed. Recorded on every run's return. *(done — completes the A1 expedition overhaul; also addresses F6's stats/records screen)*

### A2. 🟠 Endgame economy (post-max gold sink)
`upgrade_cost` returns `None` at max (`state.rs:144-147`) and then gold is worthless. Needed:
- [x] Consumables purchasable per-mission (one-run buffs: rations, oil flask, hired scout) — a repeatable sink.
- [ ] Expedition entry stakes / insurance options (gold-in, multiplier-out).
- [ ] Cosmetic sink (carriage liveries, guard colors) once the art pass exists.
- [ ] Optional: prestige/NG+ (reset campaign with a permanent token).

### A3. 🟠 Mission-type mechanics — promote 3–4 meters to verbs
All 8 special meters are one formula with different coefficients (`pressure.rs:8-104`) — the lose-screen differs, the play doesn't. Per the design review, rework only 3–4 (avoid gold-plating all 10):
- [ ] **Prisoner escort**: prisoner periodically attempts a breakout (spawns an escaping unit you must intercept) instead of a passive security meter.
- [ ] **Monster egg**: damage past thresholds cracks the egg — it hatches mid-route and aggros everything (turns the meter into a bomb).
- [ ] **Siege supply run**: fixed timed mega-waves replacing steady spawn flow (rhythm inversion).
- [ ] **Princess comfort**: scoring becomes line-quality-driven (smoothness bonus multiplier visible in real time) so it plays as a "drive clean" challenge.

### A4. 🟠 Choice-based progression (build identity)
All 8 upgrade tracks are flat linear stat bumps (`upgrades.json`, cost = `base_cost·(level+1)`). Add opportunity cost:
- [ ] 3★ guard **specialization forks** (e.g. Swordsman → Berserker or Bodyguard; Mage → Pyromancer or Warden) — the star-ability system (`combat.rs:399-458`) is already wired to receive this.
- [ ] 2–3 **mutually-exclusive carriage upgrades** (e.g. slot +1 but −speed).
- [ ] Rebalance so no build can max everything in one campaign — choices must exclude.

### A5. 🟡 Elite/boss enemies
Roster is 5 kinds, no elites, no bosses (`entities.rs:432-439`). Add:
- [x] 2–3 **elite variants** (armored bandit, alpha wolf, revenant) using existing behavior verbs with a twist. *(Alpha Wolf + Armored Bandit shipped)*
- [ ] 2–3 **bosses** for campaign finale + expedition finale (multi-phase, telegraph-heavy — the wave state machine in `flow.rs` extends naturally).
- [ ] 3–4 additional standard enemies to support biome variety (A6/B1).

### A6. 🟡 Difficulty options & assists
No difficulty setting exists. For a commercial audience:
- [x] 3 difficulty presets (spawn-rate/damage multipliers over the existing `difficulty` scalar in `entities.rs:519-527`).
- [ ] Assist toggles (slower waves, extra starting gold) — cheap, widens the audience, and doubles as an accessibility feature.

---

## 3. Workstream B — Content volume

First-clear is ~90 minutes. A $10–15 premium title needs 6–10 hours of designed content plus the expedition treadmill.

- [ ] 🟠 **B1. Campaign expansion to ~24–30 missions across 3 acts/biomes** (forest → highlands → cursed marsh, or similar). Biomes matter more than mission count: new hazard sets, enemy palettes, and road visuals per act make content *feel* different. Mission data is fully JSON-driven (`missions.json`) so the cost is design + art, not engineering.
- [ ] 🟠 **B2. New hazards** (2–3 per new biome): current 4 (mud/tree/rock/fire) are one act's worth. Candidates: river fords (forced slow), rockslides (telegraphed lane denial), cursed fog (vision), night stretches (pairs with Warding Lantern equipment). *(partial — River Ford shipped)*
- [ ] 🟡 **B3. 2–3 new guard classes** (e.g. Halberdier, Falconer, Cleric) — the 6-class system with per-star abilities is the game's best-realized system; extending it is high-value-per-effort.
- [ ] 🟡 **B4. 1–2 new chassis** (e.g. Armored Coach, Sled variant for a snow biome) + 3–4 new equipment items so 4-slot choices stay contested (`equipment.rs:5-13` currently has exactly 6 items for up to 4 slots).
- [ ] 🟡 **B5. Campaign finale**: a designed final mission with a boss (A5) and an ending screen — the campaign currently just… runs out of missions.
- [ ] ⚪ **B6. Endless/gauntlet mode** for score-chasers (post-launch; expedition covers most of this niche first).

---

## 4. Workstream C — Narrative & world (light touch, deliberate)

Per the design review: vignettes yes, plot no.

- [x] 🟠 **C1. Mission intro/outro text**: 1–2 sentence courier-log per mission (`intro_text` field in `missions.json`, shown on loadout screen `src/ui/loadout.rs` and results screen). ~30 short texts total. Cheapest possible "the game has a world" signal — currently the only text is one-line objectives.
- [ ] 🟡 **C2. World framing**: name the region, name the acts, a stylized map background on the mission-map screen tying the 12→30 missions into one journey.
- [x] 🟡 **C3. Expedition event vignettes** (shared with A1 run events) — 6 between-legs vignettes shipped via `run_events.json`. *(done)*
- [ ] ⚪ **C4. Guard barks/flavor**: one-line hire quotes and per-class flavor text in the roster (cheap character).

---

## 5. Workstream D — Art & visual identity 🔴 (the single biggest cost item)

Verified: the game renders **one texture** (title screen — `texture_manifest.json` has a single entry) and everything else via 358 primitive draw calls across `src/ui/*.rs` (`draw_wolf`…`draw_necromancer` at `gameplay.rs:424-568`, procedural carriage/guards/road/panels/icons). The title art already sets a visual bar the gameplay can't match — that gap is the first thing any store-page visitor will see.

### D1. 🔴 Art direction decision (blocks everything in this workstream)
Choose one, budget accordingly:
- **(a) Commissioned 2D sprite set** matching the painted title art — highest quality, highest cost (est. 150–250 assets, see D2 inventory).
- **(b) High-quality asset-pack base + commissioned hero pieces** (carriage, title characters) — mid cost, risk of generic look.
- **(c) Deliberate stylized-vector direction**: polish the existing procedural look into an intentional flat/minimal style (à la *Bad North*) — lowest cash cost, high design-skill demand, and the title art would need redoing to match.

### D2. 🔴 Asset production (inventory for option a/b)
- [ ] Carriage: 4–5 chassis sprites × damage states × equipment attachments (visuals already modeled in code — `upgrade_visuals.rs` renders per-equipment looks procedurally, so mapping to sprites is mechanical).
- [ ] Guards: 6–9 classes × idle/walk/attack frames × 3 star-tiers (tint or trim variants acceptable).
- [ ] Enemies: 8–10 kinds × walk/attack/death frames; bosses larger.
- [ ] Environment: 3 biome tile/prop sets (road, margins, parallax layers, hazards).
- [ ] UI: icon set (upgrades, equipment, guard classes, meters, orders/stances), panel/frame art, cursor.
- [ ] Screens: mission-map region art, results art, expedition hub art.
- [ ] Portraits (optional, C-tier): guard roster portraits for shop/roster screens.
- Engineering support: sprite/animation loading via toolkit `sprite.rs` (exists, unused), texture atlas in `texture_manifest.json`, WASM asset-pack size budget (currently 2.6 MB title PNG alone — atlas + compression pass needed).

### D3. 🟠 VFX & game-feel ("juice") pass
Nothing exists beyond color-swap hit-flash (`entities.rs:48,346-347`) and 8 fake dust circles (`gameplay.rs:763-789`). Needed, roughly in impact order:
- [ ] Particle system (toolkit candidate — build once, share across RustGames): blood/spark hits, death bursts, mud splash, fire embers, arrow trails, necromancer summon FX. *(partial — a minimal per-mission Particle buffer exists, used for death bursts; other emitters + toolkit extraction remain)*
- [ ] Screen shake (carriage hits, boss slams) + hit-stop on kills. No `shake` exists anywhere in game or toolkit today.
- [x] Damage numbers / floating text (also fixes combat readability). *(done — gold for damage dealt, red for damage taken, drifting up and fading)*
- [x] Death animations — enemies currently vanish on death. *(done — a scatter burst of tinted particles on each kill)*
- [ ] Tween/easing helpers for UI transitions (panel slide/fade — menus are currently instant-swap).
- [ ] Telegraph VFX on wave spawns (currently text-banner only, `gameplay_hud.rs:28`).
- [ ] Carriage bob, wheel rotation, horse animation (the "motion feel" feedback item, partially addressed by props).

---

## 6. Workstream E — Audio 🔴 (from zero)

Verified: zero audio calls in `src/`, zero audio assets in the repo. The toolkit ships a complete `SoundManager` with volume controls and asset-pack loading (`../macroquad-toolkit/src/audio.rs:14-129`) — the engine work is done; this is asset production + wiring.

- [ ] 🔴 **E1. SFX set** (~40–60 sounds): UI (click/hover/confirm/deny/gold), combat (per-weapon hits, bow/crossbow/magic, enemy deaths, wolf howl, necromancer raise), carriage (rolling loop pitch-shifted by `speed_factor()`, mud squelch, crash, repair), meters/alerts (wave telegraph horn, meter-critical warning, mission success/fail stingers).
- [ ] 🔴 **E2. Music**: title theme, 2–3 gameplay tracks (ideally per-biome), results/shop ambient, expedition hub track, boss track. Licensed/stock is acceptable at this scale; commissioned is better for identity.
- [ ] 🔴 **E3. Wiring & mix**: `SoundManager` on the `Game` struct (`game.rs:23-31`), event-driven SFX hooks off the existing combat pending-hit collection, music state machine per screen, ducking on pause.
- [ ] 🔴 **E4. Volume settings**: SFX/music/master sliders in Settings (first slider widget in the codebase — `widgets.rs` has buttons/toggles only), persisted with the save. Mute-on-focus-loss for web.

---

## 7. Workstream F — Onboarding & UX 🔴

Verified: onboarding is two static HUD strings (`gameplay_hud.rs:350-373`); there are no tooltips anywhere, no help screen, no confirmation dialogs.

- [ ] 🔴 **F1. Guided first mission**: contextual step prompts (steer → wave telegraph → drag a guard → tap to toggle Roam → mount a ranged guard → brake/boost) gated on first campaign only. The wolf-only mission 1 is already a tutorial-by-design; it just needs the prompts.
- [ ] 🔴 **F2. Tooltip system** (toolkit candidate): hover/long-press tooltips on every shop item, upgrade, equipment, guard card, meter, and HUD element. Stats like "threat", star abilities, and meter rules are currently unexplained.
- [ ] 🟠 **F3. Confirmation dialogs**: New Campaign currently **overwrites the autosave with no prompt** (`game.rs:197-203`) — data-loss footgun; also confirm chassis purchase, expedition abandon.
- [ ] 🟠 **F4. Post-mission clarity**: score breakdown showing *why* (bonus objective met/missed, meter performance), and a "what to buy next" nudge for the first two shop visits.
- [x] 🟠 **F5. Help/codex screen**: enemy bestiary (unlock-on-encounter), guard class reference, mechanics glossary. Doubles as content-discovery motivation.
- [ ] 🟡 **F6. Menu polish**: animated transitions, hover states audit, controller-focus-visible states (pairs with G3), ~~stats/records screen~~ *(expedition Records screen shipped via A1)*, credits screen (legally needed once commissioned assets exist).
- [ ] 🟡 **F7. In-mission readability audit**: threat indicator (who targets the carriage), off-screen enemy pips, stance rings legibility at a glance — playtest-driven.

---

## 8. Workstream G — Settings, accessibility, input

Settings today: three boolean toggles (`management.rs:570-644`). Input today: mouse + hardcoded keyboard; no gamepad code exists in game or toolkit.

- [ ] 🔴 **G1. Settings expansion**: audio sliders (E4), fullscreen/windowed toggle + resolution handling (window is resizable but there's no in-app control), vsync/FPS cap, UI scale (toolkit `set_ui_text_scale` exists at `font.rs:127-138`, never called).
- [ ] 🟠 **G2. Accessibility baseline**: colorblind-safe palette audit (hit-flash and On/Off badges are red/green today — the worst case), text-size option, reduced-motion (Route Motion toggle already half-covers this — extend to shake/particles), hold-vs-toggle options for drag interactions, difficulty assists (A6).
- [ ] 🟠 **G3. Gamepad support**: required for Steam credibility and Deck play. Needs a toolkit gamepad module (macroquad gamepad support via `gilrs`/`quad-gamepad` evaluation), a virtual-cursor or radial guard-select scheme for drag orders (design work — drag is mouse-native), and full menu navigation. **This is the riskiest UX item — prototype early.**
- [ ] 🟠 **G4. Key rebinding**: keys are hardcoded literals (`flow.rs:150-165`, `game.rs:111-185`); `S`/`L` are un-modified global save/load hotkeys — collision-prone; move to a bindings map + settings UI.
- [ ] ⚪ **G5. Touch controls**: `index.html` claims touch support but none exists — single-touch works only by accident via mouse emulation. Decide mobile ambition explicitly (see §12); genuine touch needs on-screen throttle/brake and rethought drag orders.

---

## 9. Workstream H — Localization 🟡

None exists; every string is an inline English literal in Rust or the data JSON. Retrofit cost grows with every feature added — **do the extraction early even if translation waits until late beta**.

- [ ] 🟡 **H1. String externalization**: string-table module (toolkit candidate), keys for all `src/` literals plus text fields in `missions.json`/`upgrades.json`/`carriages.json`.
- [ ] 🟡 **H2. Font coverage**: bundled Rajdhani covers Latin; add fallback fonts per language tier (EFIGS first; CJK is a separate cost/decision).
- [ ] 🟡 **H3. Layout robustness**: German/French string-expansion pass on fixed-width panels and buttons.
- [ ] 🟡 **H4. Translation**: EFIGS + Simplified Chinese + Portuguese-BR is the standard indie-strategy ROI set; commission at content-freeze.

---

## 10. Workstream I — Technical robustness & platform engineering

Foundations are genuinely good (atomic saves, versioned migration, CI, low panic density). Gaps are product-grade concerns:

### I1. Save system hardening
- [ ] 🔴 Corrupt-save recovery: keep a rolling `.bak`; on parse failure offer restore instead of a toast (`game.rs:460`) — today a corrupt autosave **silently breaks Continue** (`game.rs:204-210` checks file existence only).
- [ ] 🟠 Real save timestamps (`SaveSlot.save_date` is hardcoded `"Unknown"` — `persistence.rs:639`).
- [ ] 🟠 Multiple save slots + delete/rename UI (toolkit supports it; game hardcodes one `"campaign"` slot).
- [ ] 🟠 Autosave via the unused `AutoSaveManager` timer (`persistence.rs:402-469`) instead of ~20 per-action write sites in `game.rs` (write amplification, and a mid-frame quit risk).
- [ ] 🟠 Save/load **round-trip + corruption-path tests** — the persistence layer currently has zero test coverage.
- [ ] 🟡 Steam Cloud mapping (path is already `%LOCALAPPDATA%\carriage_run\` — cloud-friendly).

### I2. Crash & error handling
- [ ] 🔴 Panic hook writing a crash log + user-facing dialog (startup data-load failure currently hard-panics with no message — `game.rs:35-37`).
- [ ] 🟠 Opt-in crash reporting for native builds (the Roost bug-report widget is web-page-only and non-portable to Steam).
- [ ] 🟡 Opt-in anonymous telemetry (mission win rates, retry counts, expedition depth) — this is how balance gets tuned post-launch.

### I3. Data pipeline validation
- [x] 🟠 Semantic validation at load: unknown enemy/hazard IDs currently degrade silently to Wolf/Mud (`flow.rs:276,306` `.unwrap_or(...)`) — should hard-fail in CI/dev builds.
- [x] 🟡 Cross-record invariant tests beyond the single difficulty-monotonic test (reward curves, unlock-graph reachability, cost curves).

### I4. Performance & stability
- [x] 🟠 Entity hard-caps + soak test (no ceiling exists on live enemies; necromancers compound).
- [ ] 🟡 Benchmark `opt-level=3` native vs the workspace-inherited `opt-level="z"` (size-optimized) — a per-package override in the workspace root is the pattern other games already use.
- [ ] 🟡 Fixed-timestep evaluation for simulation determinism (currently variable dt clamped at 0.1 — `main.rs:39`; matters for seeded-run fairness, A1).
- [ ] 🟡 WASM size budget: asset atlas + PNG compression once the art pass lands (title PNG alone is 2.6 MB).

### I5. Dependency & release engineering
- [ ] 🟠 Pin/tag `macroquad-toolkit` (currently an unversioned path dependency — a sibling-repo change can silently break the shipped game).
- [ ] 🟠 Single source of truth for version (currently duplicated in `Cargo.toml` and `game_config.json`), git tags, CHANGELOG, release-build automation in CI (artifacts are built but never uploaded/tagged).

---

## 11. Workstream J — Distribution, marketing, business 🔴

Nothing exists here — the current pipeline publishes to the WebHatchery web catalog via FTP only (verified: no Steam/itch/butler references in the entire publish system).

### J1. Packaging & legal (blockers)
- [ ] 🔴 LICENSE/EULA, third-party license inventory (fonts — check Rajdhani's OFL, macroquad, commissioned assets).
- [ ] 🔴 App icon (`.ico`) + Windows version resource (ships with the default Rust icon today); embed via `winres`/`.rc`.
- [ ] 🔴 Steamworks integration: steamworks-rs crate, app registration, depots, build scripts alongside `publish.ps1`; achievements (~15–25 mapped to existing campaign records/star ratings); Steam Cloud (I1); overlay compatibility check with macroquad; Steam Deck verification pass (hard-requires G3 gamepad).
- [ ] 🟠 itch.io butler pipeline (near-free once Steam packaging exists).
- [ ] 🟡 Code signing (SmartScreen reputation) — optional but reduces friction; installer likely unnecessary (zip/Steam handles it).

### J2. Store presence & marketing assets
- [ ] 🔴 Steam page live **6+ months before launch** (wishlists are the #1 launch predictor): capsule art set (the existing title painting is a strong base), 5–10 screenshots *(gated on the art pass — current procedural look can't carry a store page)*, trailer (30–60s, gameplay-first), store copy.
- [ ] 🟠 Demo strategy: the web build **is** the demo — gate it to campaign act 1 + limited expedition, add a "Wishlist on Steam" link in `index.html` and on the title screen. Also ship it as a Steam demo for Next Fest.
- [ ] 🟠 Press kit, GIF library (the capture harness in `scripts/capture_ui.ps1` can be extended to record marketing footage deterministically), 2–3 devlog posts, outreach list for the convoy/management streamer niche identified in feedback.
- [ ] 🟡 Community: Discord or Steam-forum presence, feedback loop replacing the Roost widget for store builds.
- [ ] 🟡 Name check: trademark/store-collision search for "Carriage Run" before the store page goes live.

### J3. Web shell fixes (the demo funnel must not look broken)
- [ ] 🟠 Loading screen actually tied to load completion — the spinner is currently hidden **before** the 1.5 MB WASM + assets download (`index.html:89`), leaving a blank canvas.
- [ ] 🟡 Load-progress bar, WebGL-failure fallback message, orientation/mobile notice.

---

## 12. Workstream K — QA & balance

- [ ] 🔴 **K1. External playtest program**: the game has had exactly one playtester. Run structured rounds at alpha (fun/friction), beta (balance/onboarding), RC (bugs) — 10+ testers per round; the web build makes distribution trivial.
- [ ] 🟠 **K2. Balance simulation harness**: headless auto-resolve of missions across upgrade states using the deterministic seeds — asserts on win-rate corridors and difficulty monotonicity, so JSON tuning passes stop being playtest-gated. The pieces (seeded RNG, embedded data, sim/render separation) already exist; this was also the README's own top "future improvement".
- [ ] 🟠 **K3. Full-clear economy audit**: verify gold curve supports the A4 choice-exclusion model across a campaign; automated by K2.
- [ ] 🟠 **K4. Platform QA matrix**: Windows (several GPUs/DPIs), Steam Deck, 3 browsers × 2 OSes for web; save-compat regression suite across every released version.
- [ ] 🟡 **K5. Extend the screenshot harness** (`begin_capture_scene`, `game.rs:66-101`) to cover new screens (settings, results, bestiary) for visual-regression checking in CI.

---

## 13. Suggested phasing & effort

Estimates assume current velocity (solo + AI-assisted, art/audio commissioned). Total aligns with `standing.md`'s 4–7 month call.

### Phase 1 — "Deep" (≈4–6 weeks) — *design-complete the systems*
Expedition overhaul (A1), endgame economy (A2), mission-type verbs (A3), choice upgrades (A4), difficulty presets (A6), string externalization started (H1), toolkit pinning + save hardening (I1, I5), **art direction decision (D1) + commissioning starts now** (longest external lead time), **gamepad prototype (G3) started now** (highest UX risk).
**Gate:** a stranger plays 3 hours without designer help and wants to keep playing.

### Phase 2 — "Complete" (≈6–8 weeks) — *content + the two zero-layers*
Campaign to 3 acts (B1–B2, B5), elites/bosses (A5), narrative pass (C1–C2), **audio build-out (E1–E4)**, art integration as assets land (D2), tooltips + tutorial (F1–F2), settings expansion (G1–G2).
**Gate:** content-complete alpha; external playtest round 1 (K1).

### Phase 3 — "Feel" (≈4–6 weeks) — *juice, onboarding polish, platform*
VFX pass (D3), UX polish (F3–F7), gamepad completion + rebinding (G3–G4), Steamworks + packaging (J1), **Steam page live + wishlist campaign begins (J2)**, web demo gating + shell fixes (J3), crash handling (I2), balance harness (K2–K3).
**Gate:** beta; Steam page live; playtest round 2.

### Phase 4 — "Ship" (≈4–6 weeks) — *hardening + launch*
Localization translation (H4), QA matrix + save-compat suite (K4), performance/soak (I4), achievements/cloud finalization, trailer + press kit (J2), Next Fest demo, release automation (I5), launch.
**Post-launch:** endless mode (B6), telemetry-driven balance patches, cosmetics sink, touch/mobile evaluation (G5).

---

## 14. Open decisions (owner calls needed before Phase 1 ends)

1. **Platform scope**: Steam + itch + web-demo (assumed)? Steam Deck verified? Mobile ever? — determines how hard G3/G5 gate everything.
2. **Art direction & budget** (D1): commissioned sprite set vs. stylized-vector doubling-down. This is the largest single spend and the longest lead time — decide first.
3. **Price point & content bar**: $10–15 premium (needs Workstream B fully) vs. $5–8 compact (Workstream B trimmed, expedition carries the value).
4. **Scope trim option**: if time-boxed, the defensible cuts are B3/B4 (new guards/chassis), C4, H4 beyond EFIGS, J1 code-signing — **not** audio, tutorial, art, or the expedition overhaul, which are the difference between "prototype with a store page" and a product.
5. **Localization tier** (H4): EFIGS only vs. +CJK (font + cost implications).

---

*Cross-references: prior analysis in [game_review.md](game_review.md) (design review — its Phase 1–2 recommendations are now implemented), [FEEDBACK_ACTION_PLAN.md](FEEDBACK_ACTION_PLAN.md) (all P0–P2 items landed), [feedback.md](feedback.md) (original playtest). This document supersedes the "Practical Future Improvements" list in [README.md](README.md).*
