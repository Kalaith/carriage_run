# Carriage Run — Design Review

*Senior design / production / systems review. Reviewed from source, assets, feedback notes, and the two verification screenshots — the game was not played live. Where I make a claim about feel I flag it as inference.*

---

# 1. Project Overview

## Project Name
**Carriage Run**

## Genre
Reverse tower-defense / real-time escort strategy. Single-player, run-based, campaign-structured. Rust + macroquad (native Windows + WebGL).

## Core Concept
The player is a **caravan master**. Each mission is a top-down, vertically-scrolling road: the carriage rolls forward automatically toward an exit while the player (a) steers it left/right within the road to dodge hazards and (b) commands a small squad of hired guards via drag orders to intercept wolves, bandits, archers, skeletons, and necromancers before they reach the cargo.

- **What the player does:** steer the carriage, position/redirect guards in real time, and — between missions — hire guards, upgrade them (star levels), buy carriage upgrades, equip carriage systems, and choose a route branch that trades distance vs. difficulty vs. reward.
- **The fantasy:** "get the goods through." It's the wagon-train-under-attack fantasy — Oregon Trail's tension without the resource-attrition management, closer to *They Are Billions* framing inverted (you are the thing being swarmed, and you're moving).
- **What makes it different:** it inverts tower defense — *you* are the moving target and your "towers" (guards) are mobile units you micro. The dual-axis of **dodging hazards while managing combat while watching the exit approach** is a genuinely distinct tension, and the reviewer's own feedback ("felt like you are trying to avoid all the dangers while looking how close the exit was") confirms that tension lands.
- **Target player:** fans of light RTS/auto-battler hybrids, roguelite-run players, and — per the author's own note — the "management/convoy" streamer niche. Session length is short (missions are ~60s–90s of travel), which suits mobile-ish / stream-friendly play.

## Current State
**Feature-complete MVP, mid-polish — content-expansion-ready.** Not a prototype. Evidence:
- ~9,500 lines of Rust across ~30 well-structured source files; multiple files near the 800-line limit (`gameplay.rs` 792, `state.rs` 715, `combat.rs` 705, `mission_map.rs` 698, `gameplay_hud.rs` 686).
- 12 missions with 24 route branches, 10 distinct mission-type flavors, 6 guard classes, 5 enemy types, 4 hazard types, 6 upgrade tracks, 4 equipment slots, a full shop/roster/upgrade/settings/pause UI, save/load, star ratings, and campaign records.
- It has already absorbed **one full feedback round**: the [feedback.md](feedback.md) → [FEEDBACK_ACTION_PLAN.md](FEEDBACK_ACTION_PLAN.md) loop is done for P0/P1. Balance is rebalanced to a monotonic difficulty curve, a speed gauge ("18 mph CRUISE") is on the HUD, roadside props scroll for motion, and the route map hides undiscovered missions behind "???" teasers.

This is a game that works and has been iterated once. The open questions are about **depth and retention**, not viability.

---

# 2. Core Gameplay Analysis

## Main Gameplay Loop

**Micro loop (in-mission, ~60–90s):**
> Watch road → steer around hazard → spot incoming enemy → drag a guard to intercept → return guard to escort → check exit distance → repeat → arrive → score.

**Macro loop (between missions):**
> Earn gold → hire/upgrade guards + carriage → pick next route branch → equip loadout → run mission → unlock next tier → repeat.

## Evaluation

- **Clear?** Yes. The dual objective (deliver cargo + keep carriage alive) is legible, and the HUD shows health/cargo/progress/speed. The route-choice screen exposes distance/difficulty/reward/threats/hazards up front.
- **Satisfying?** The core tension is validated by the playtester. The satisfaction *ceiling* is currently capped by how thin the **moment-to-moment decision-making** is — see below.
- **Meaningful decisions?** The best decision in the game is the **route branch** (safe-long-cheap vs. fast-hard-lucrative) — this is genuinely good and underexploited. In-mission, the decision density is lower: most guards on Escort auto-attack, so the player's active input is often just steering plus occasional drag-redirects. The reviewer intuited this exactly ("maybe a free roam option for the guards... and direct when needed") — they're asking for **fewer forced micro-actions**, which is a signal the micro-loop is either too fiddly or too shallow depending on the moment.
- **Variety?** Good breadth on paper (12 missions, 5 enemies, 6 guards) but the enemies are largely stat reskins of two behaviors (melee-rusher / ranged-poker) plus the necromancer's raise-skeleton wrinkle. Mission *types* (prisoner, princess, time-delivery, siege) are mostly reward/objective-text variations rather than mechanically distinct rules.
- **Long-term motivation?** This is the weakest axis. Progression is a linear unlock ladder gated by carriage level + prerequisites. Once you've cleared the 12 missions, there's no reason to replay them (missions are fixed, no scaling/endless mode, no meta-goal beyond star ratings — deterministic seeds exist under the hood but aren't surfaced as a replay feature). Retention past ~1 hour is unaddressed.

**Verdict:** Strong, distinctive core; shallow decision density in-mission; thin long-term hook. The loop is *fun-capable* but currently *content-limited* and *depth-limited*.

---

# 3. Existing Systems Review

## Carriage Steering & Movement
### Purpose
The player's direct, twitch-level agency — dodging hazards and repositioning under fire.
### Current Implementation
Carriage is fixed at `CARRIAGE_Y = 506`; the player sets a `target_x` and the carriage eases toward it with weighted (inertial) horizontal movement, clamped to a curving road (`road_curve_offset` gives a sine-based serpentine road). Mud applies a `slow_timer`; hazards (fallen tree, rocks, fire) deal damage/cargo loss. Speed derives from `scroll_speed()`/`speed_factor()`, now surfaced on the HUD.
### Strengths
Weighted movement gives the carriage a believable heft. The curving road plus the new scrolling roadside props sell motion (the feedback's "ground vs. caravan" complaint was addressed). Speed gauge with cruise/slow states is a clean, readable addition.
### Weaknesses
Steering is a single axis (left/right). Once you internalize hazard lanes it becomes rote. The road-curve is cosmetic — it doesn't create positional decisions (you can't take an inside line for advantage).
### Improvement Ideas
Make the road *shape* matter: narrow chokepoints that force you to slow or brush hazards; forks you physically steer into (tie the route-branch choice to an in-mission fork you drive through rather than a menu). Add a **manual brake/boost** so speed becomes a player-controlled resource (slow to let guards catch up, boost to sprint an exposed stretch) — this instantly deepens the micro-loop.
- **Impact: High** · **Cost: Medium**

## Guard Command & Combat
### Purpose
The strategic layer — the "reverse tower defense" heart of the game.
### Current Implementation
6 guard classes (Swordsman/ShieldGuard/Spearman melee; Archer/Crossbow/Mage ranged). Ranged guards can **mount** carriage slots. Orders: `Escort` (auto-attack in range while holding formation), `Move(pos)`, `Hold`, `Attack(enemy_id)` — issued via drag. Guards have HP, speed, range, armor, cooldown, star levels (1–3) with ability text. Enemies: Wolf, Bandit (steals cargo), BanditArcher (ranged), Skeleton, Necromancer (raises skeletons). Combat resolves via pending-hit collection each frame.
### Strengths
The mount/dismount mechanic for ranged guards is a nice spatial choice (mobile turret vs. ground skirmisher). Class identities are distinct on paper (shield = puller/tank, spear = anti-charge reach, mage = splash). Necromancer's raise mechanic is the one enemy that creates an emergent priority-target decision — kill the caster or drown in skeletons.
### Weaknesses
- **The `Roam` gap.** There is no self-directing stance. `GuardOrder` is only `Escort/Move/Hold/Attack`. Escort auto-attacks *within range* but won't proactively intercept, so the player must babysit with drags — exactly the friction the tester flagged. This is the single most important combat fix and it's specced in the action plan but **not yet implemented**.
- Enemy behavior variety is low: most are melee-rushers or ranged-pokers with different stats. The AI doesn't flank, focus-fire the carriage vs. guards intelligently, or create readable "waves."
- Combat readability under load is a risk (inference): with 4+ enemies and a squad, the drag-order model may get frantic — the tester's request for automation supports this.
### Improvement Ideas
1. **Implement `GuardOrder::Roam`** (leash-radius auto-intercept, direct orders override) and consider making it the default melee stance. *This is the highest-leverage single change in the game.*
2. Give each enemy one **verb**, not just a statline: wolves flank in packs, bandits make a dash for cargo then flee (recoverable if you kill them), archers kite to max range, necromancers hang back. This turns combat from stat-check into a read.
3. Telegraphed **waves** with a brief lull between them, so the player gets rhythm and breathing room (also solves route-start pressure).
- **Impact: Game-changing** · **Cost: Medium** (Roam) / **Large** (full enemy-verb pass)

## Progression & Economy
### Purpose
Between-mission growth and the reason to keep clearing missions.
### Current Implementation
Gold from missions (base_reward ± route delta), starting 125. Spend on: hiring guards (Shield 120 / Spear 170 / Crossbow 135 / Mage 190; Swordsman & Archer free), star upgrades (90–240), and 6 carriage upgrade tracks (armor, guard_training, mounted slots, wheels, cargo straps, repair kit; base_cost 45–80, max level 3–4). Carriage level gates mission unlocks and equipment slot count (2/3/3/4).
### Strengths
Multiple spend targets create real budgeting tension early (the tester noted ~100 gold buys only 2–3 upgrades entering map 2 — that scarcity *is* a decision). Route deltas let a confident player gamble for more gold. Free starter guards mean no dead-start.
### Weaknesses
- Upgrades are **flat linear stat bumps** — "Iron Plating +HP", "Guard Training +dmg". None change *how* you play. There are no build-defining choices, no synergies, no trade-offs (every upgrade is strictly good, so the only question is order-of-purchase).
- The economy is a **one-way ratchet**: you always get richer, upgrades are permanent, and there's no sink after the tree is maxed. Post-max, gold is meaningless and so are rewards.
- Carriage level is doing three jobs (mission gate, slot count, plating level) — overloaded and hard to reason about.
### Improvement Ideas
1. Introduce **mutually-exclusive or opportunity-cost choices**: an upgrade that adds a slot but reduces speed; a guard specialization fork (a Swordsman can go Berserker *or* Bodyguard at 3-star). Choice creates replay identity.
2. Add the **carriage chassis system** from the action plan (Scout/Standard/Heavy with different slots/speed/HP) — this is the tester's explicitly requested "next milestone" and it converts carriage level from a number into a *decision*.
3. Add a **gold sink with stakes**: consumables (one-run buffs), guard revival costs, or route tolls.
- **Impact: High** · **Cost: Medium–Large**

## Route Choice / Campaign Structure
### Purpose
The map-level strategic decision and pacing gate.
### Current Implementation
12 missions on an unlock graph (level gates + prerequisite missions + "unlock_any" branch gates). Each mission has 2 route branches with distance/difficulty/reward/time deltas and enemy/hazard adds. Undiscovered missions now hide behind "???" teasers (near-unlock detection via `is_mission_near_unlock`).
### Strengths
**This is the strongest system in the game.** The safe-vs-greedy branch choice is a clean, repeatable, meaningful decision, and the teaser-fog (recently added) fixed the "too much info" complaint while preserving a sense of a road ahead. The difficulty curve is now cleanly monotonic (1.00 → 1.80).
### Weaknesses
The route choice is a **menu decision made once before the mission** — it doesn't recur or create in-mission consequence beyond the enemy/hazard mix. Branches don't visibly diverge the world (no persistent map). With only 12 fixed missions and a linear ladder, the campaign is a ~1–2 hour experience with no replay driver.
### Improvement Ideas
1. Turn the branch into an **in-mission fork you steer into** (see Steering) — makes the choice visceral and repeatable.
2. Add a **light run/meta structure**: a randomized or semi-randomized "journey" that chains missions with escalating stakes and a fail-state, giving a reason to replay with different loadouts (roguelite convoy run).
- **Impact: High** · **Cost: Medium**

## Mission Types & Objectives
### Purpose
Content variety and flavor.
### Current Implementation
10 named mission types (cargo_transfer ×3, time_delivery, prisoner_escort, medicine_run, gold_shipment, monster_egg_transport, refugee_escort, princess_escort, royal_banquet_supplies, siege_supply_run). Each has objective + bonus objective text, cargo flavor, and (some) time limits. Missions also carry "security/comfort" meters per the README.
### Strengths
Strong *flavor* breadth — the names promise variety and set up a story world cheaply.
### Weaknesses
Each type *does* carry a unique **failure meter** (`special_meter`: prisoner security, princess comfort, medicine potency, egg stability, refugee safety, banquet freshness, siege momentum, plus deadlines) — so they're more than pure flavor. But those meters are mostly **fail-condition wrappers**: they change *what you lose* if you take too many hits, not *how you steer or command*. The player's moment-to-moment actions are nearly identical across a "prisoner escort" and a "supply run." The distinctiveness is in the lose-screen, not the play.
### Improvement Ideas
Promote the meters from passive fail-states to **active mechanics** that alter play: prisoner = the cargo periodically tries to *escape* (a breakout unit you must intercept, not just a meter); princess comfort = rewards a smooth, hazard-free line, creating a genuine speed-vs-safety tension distinct from combat; monster egg = if damaged it *hatches and aggros* enemies mid-route; siege = timed waves instead of steady travel. You don't need all 10 reworked — turning 3–4 meters into verbs is plenty.
- **Impact: High** · **Cost: Medium**

## UI / UX
### Purpose
Legibility of a real-time, multi-object game.
### Current Implementation
Full suite: routes map, shop, guards roster, upgrades, settings, in-mission HUD (health/cargo/progress/timer/repair/pause), guard cards with drag-to-reposition, route detail panel with threats/hazards/route-choice. Speed gauge added.
### Strengths
Genuinely polished for an MVP — the screenshots read as a shippable indie UI, not a prototype. Good information hierarchy on the route screen.
### Weaknesses
- Several UI files are near the 800-line hard limit and will need extraction as UI grows (`mission_map.rs` 698, `gameplay_hud.rs` 686, `management.rs` 665).
- (Inference) In-mission, guard status + orders + enemy threats + hazards + steering is a lot of simultaneous reading; the drag-order model competes with steering for the same cursor/attention.
### Improvement Ideas
Guard stance icons + click-to-cycle (from action plan) to reduce drag reliance; a subtle enemy-threat indicator (who's targeting the carriage). Keep watching file sizes.
- **Impact: Medium** · **Cost: Small–Medium**

---

# 4. Similar Games & Lessons

## Bad North
Minimalist real-time squad-command tactics on tiny islands: you drag a handful of units to defend against waves. **Does better:** unit stance/auto-engage so you're making *positional* decisions, not babysitting attacks; crystal-clear readability; permadeath stakes. **Adapt:** the auto-engage leash (this is exactly the `Roam` stance) and the "few units, high-clarity, high-stakes" philosophy. **Don't copy:** its static-island structure — Carriage Run's *movement* is its differentiator.

## They Are Billions / Kingdom (Two Crowns)
Kingdom is the closest cousin: a side-scrolling, you're-a-mobile-base-under-nightly-assault economy game. **Does better:** the tension of a *moving frontier* and a day/night rhythm; every coin spent is a visible defensive choice; gorgeous minimal art selling motion. **Adapt:** rhythm/pacing (waves with lulls), and making economy spends *visible* on the carriage. **Don't copy:** its slow burn — Carriage Run's short missions are an asset for streaming/mobile; don't bloat them.

## FTL: Faster Than Light
The gold standard for "run-based journey with route choice under threat." **Does better:** the *map is the game* — a branching journey where every node is a gamble (fight/reward/risk), a pursuing threat forces forward motion, and permadeath makes each choice matter. **Adapt:** the roguelite-journey meta-structure is the single best template for Carriage Run's retention problem. Chain missions into a run with escalating pressure and a fail-state. **Don't copy:** FTL's punishing opacity — keep Carriage Run's up-front route transparency.

## Oregon Trail / Overland
Convoy survival with attrition and hazard events. **Adapt:** the *journey-as-story* framing and event vignettes (cheap narrative). **Don't copy:** pure attrition/RNG death — the tester wants skill-expressive avoidance, not dice.

---

# 5. Feature Improvement List

## Critical Improvements
| Priority | Feature | Description | Player Benefit | Dev Cost |
|---|---|---|---|---|
| Critical | Guard `Roam` stance | Self-directing leash-radius auto-intercept; direct orders override. Default for melee. | Removes the #1 friction (babysitting); makes combat about positioning, not clicking. | Medium |
| Critical | Enemy behavior verbs | Give each enemy one distinct behavior (pack-flank, cargo-dash-and-flee, kite, raise). | Turns combat from stat-check into a readable, decision-rich fight. | Medium |
| Critical | Retention meta-loop | A roguelite "journey" chaining missions with escalating stakes + a fail-state. | The missing reason to keep playing past ~1 hour. | Large |

## High Value Improvements
| Priority | Feature | Description | Player Benefit | Dev Cost |
|---|---|---|---|---|
| High | Carriage chassis system | Purchasable chassis (Scout/Standard/Heavy) trading slots/speed/HP. | Tester's stated next milestone; converts a stat into a build choice. | Large |
| High | Manual brake/boost | Player-controlled speed as a resource. | Deepens the micro-loop; adds a lever for pacing combat. | Medium |
| High | Distinct mission-type rules | 4 of the 12 types get a unique mechanic (prisoner escape, egg hatch, siege waves, comfort fail). | Makes the varied names actually play differently. | Medium |
| High | Choice-based upgrades | Replace some flat bumps with opportunity-cost / fork upgrades. | Build identity and replay variety. | Medium |

## Nice To Have
| Priority | Feature | Description | Player Benefit | Dev Cost |
|---|---|---|---|---|
| Nice | Guard stance icons + click-cycle | Reduce reliance on drag orders. | Lower cognitive load in-mission. | Small |
| Nice | Wave telegraphs + lulls | Signal incoming groups; brief breathing room. | Rhythm, fairness, readability. | Small |
| Nice | Story vignettes (`intro_text`) | 1–2 line courier-log per mission. | Cheap connective flavor (tester noted no story). | Small |
| Nice | Guard permadeath option / injury stakes | Meaningful loss on defeat (currently recover). | Raises stakes; supports roguelite mode. | Medium |
| Nice | Wheel dust / carriage bob particles | Extra motion + speed feedback. | Polish. | Small |

## Avoid / Do Not Add
| Priority | Feature | Description | Why avoid |
|---|---|---|---|
| Avoid | Multiplayer / co-op | — | Enormous cost, dilutes the single-player micro-loop; no evidence of demand. |
| Avoid | Deep crafting / inventory economy | — | Adds management overhead orthogonal to the action-strategy core; the `actions.json` (gather/craft/research) is leftover template boilerplate — do not build on it. |
| Avoid | Free-roam / open world | — | The forward-motion pressure IS the game; open exploration kills the tension the tester praised. |
| Avoid | Full branching narrative | — | Over-investment; light vignettes are enough. Keep the budget on systems. |
| Avoid | Making every mission-type unique | — | Diminishing returns; 4 distinct rules is enough — don't gold-plate all 10. |

---

# 6. Missing Gameplay Elements

## A self-directing guard stance
**Expected?** Yes — every squad-tactics game has auto-engage. **Needed?** Yes, critically. **Implementation:** `GuardOrder::Roam` (specced in action plan). **Priority: Critical.**

## A retention / fail-state loop
**Expected?** Run-based games live or die on this. **Needed?** Yes — it's the game's biggest gap. **Implementation:** roguelite journey chaining missions; losing the carriage ends the run. **Priority: Critical.**

## Meaningful loss / stakes
**Expected?** Escort games gain tension from what you can lose. Currently guards recover after defeat and cargo is the only loss vector. **Needed?** Yes, moderately — some persistent consequence sharpens every decision. **Implementation:** guard injury downtime (bench for N missions), a run-ending carriage loss, or cargo value affecting score meaningfully. **Priority: High.**

## Build variety / synergy
**Expected?** Progression games are expected to enable different playstyles. Currently all upgrades are flat and additive. **Needed?** Yes — it's the difference between "grind the tree" and "build a caravan." **Implementation:** choice upgrades + chassis. **Priority: High.**

## Distinct enemy behaviors
**Expected?** Yes — 5 enemy names imply 5 threats; currently ~2 behaviors. **Needed?** Yes. **Priority: Critical** (bundled with combat).

## Story / world connective tissue
**Expected?** Lightly — the tester noticed its absence but didn't demand a plot. **Needed?** Low. **Implementation:** per-mission vignette text. **Priority: Low.**

*Deliberately NOT missing: multiplayer, crafting, open world, dialogue trees — none serve the core and all would dilute it.*

---

# 7. Content & Replayability Analysis

**Current replay drivers:** star ratings (3-star each mission), route-branch experimentation, unlocking the full guard/upgrade tree. That's roughly **1–2 hours of first-clear content and near-zero reason to return** afterward.

- **Variety:** broad-but-shallow. 12 missions / 24 branches / 6 guards looks like a lot, but the enemy behavior space and upgrade space are narrow, so runs feel similar.
- **Progression:** linear, permanent, one-way. Satisfying on the way up, inert at the top.
- **Randomness:** low. Missions are fixed and use **deterministic seeding** (seed derived from mission order + completion count + route choice). This *helps* skill expression and reproducibility (and quietly makes a seeded "daily run" / replay-sharing feature cheap to expose) but *hurts* replay as-is — there's nothing new on a second run.
- **Player choice:** concentrated in route branches (good) and loadout (decent); thin in-mission.
- **Emergent gameplay:** the one bright spot is the necromancer-raise priority decision — proof the design *can* generate emergence when enemies have verbs. Lean into this.
- **Long-term goals:** none beyond 100%-ing stars.

**Highest-leverage retention moves (in order):**
1. **Roguelite journey mode** — randomized mission chains + fail-state + escalating difficulty. Converts fixed content into replayable runs. *This is the single biggest retention lever.*
2. **Enemy verbs + Roam stance** — makes each fight a fresh read rather than the same stat-check.
3. **Build variety (chassis + choice upgrades)** — gives a reason to run again *differently*.
4. **Endless / gauntlet mode** — a scaling survival run for score-chasers and streamers (matches the stated audience).

---

# 8. Player Experience Review

## First 10 Minutes
The player understands the pitch immediately: steer the carriage, protect it, reach the exit. The route screen clearly frames the safe-vs-greedy choice. The first mission (wolf-only, mud-only, difficulty 1.0) is a clean tutorial-by-design. **Improve:** an explicit first-time prompt teaching drag-orders and mounting (currently discoverable but unguided — inference); surface the bonus objective more prominently so players learn to play for score early.

## First Hour
The hook is present — the tester confirmed the danger-vs-exit tension works and progression through the early missions feels good after the rebalance. The risk in this window is the **micro-loop fatiguing** (constant guard-babysitting without Roam) and the **upgrade choices feeling samey** (flat bumps). Landing the Roam stance and one or two choice-upgrades would carry the first hour comfortably.

## Long-Term
Currently unaddressed. After clearing 12 missions the player has no loop. This is the make-or-break gap: without a journey/endless/build-variety layer, Carriage Run is a polished ~90-minute experience, not a game people return to. Everything in Phase 2–3 below targets this.

---

# 9. Development Roadmap

## Phase 1 — Make It Fun (tighten the core)
**Goals:** eliminate the babysitting friction and deepen the micro-loop so 60 seconds of gameplay is consistently engaging.
- Implement `GuardOrder::Roam` (default melee stance) + stance UI.
- Give each enemy one distinct behavior verb.
- Add manual brake/boost.
- Add wave telegraphs + brief lulls.
**Why first:** these fix the exact friction the tester named and raise the fun ceiling of the second-to-second experience — no point adding content on top of a loop that fatigues.

## Phase 2 — Add Depth (build identity + stakes)
**Goals:** make progression a series of decisions, not a ratchet.
- Carriage chassis system (Scout/Standard/Heavy) — the tester's requested milestone.
- Choice-based / fork upgrades and 2–3 new equipment items.
- Meaningful loss (guard downtime or run fail-state).
**Why second:** depth is only worth adding once the core is fun; these systems also set up the meta-loop.

## Phase 3 — Add Content & Retention (reason to return)
**Goals:** convert fixed content into replayable runs.
- Roguelite journey mode: randomized mission chains, escalating difficulty, fail-state.
- 4 mission types get unique rules.
- Optional endless/gauntlet mode for score-chasers/streamers.
**Why third:** retention systems need the deepened core + builds to be worth replaying — otherwise you're randomizing a shallow loop.

## Phase 4 — Polish
**Goals:** ship-quality feel.
- Particles (dust/bob), audio pass, first-time tutorial prompts, story vignettes.
- File-size hygiene: extract the UI files nearing 800 lines.
- Balance pass on new systems; regression tests for the difficulty curve (already partly specced).
**Why last:** polish amplifies a good game; applied earlier it amplifies gaps.

---

# 10. Final Assessment

## Strongest Idea
The **moving-target reverse tower defense**: dodging hazards while commanding a mobile squad while watching the exit close. It's a genuinely distinctive tension, it's validated by the playtester, and the route-branch risk/reward layered on top is a clean, repeatable strategic decision. Protect this core — it's the reason the game exists.

## Biggest Risk
**Retention collapse.** The game is a polished ~90-minute experience with a linear, one-way progression and no reason to return. Combined with the shallow in-mission decision density (guard babysitting, flat upgrades, stat-reskin enemies), the risk is that players enjoy it, finish it, and never reopen it — fatal for a title aimed at the streamer/replay niche.

## Missing Ingredient
**A run-based meta-loop with stakes** (roguelite journey + fail-state). One structural addition converts all the existing fixed content into replayable runs and gives the permanent-progression tree something to feed. If only one thing gets built, build this — with the `Roam` stance as its non-negotiable prerequisite for the core to feel good.

## Unique Selling Point
"Tower defense, but *you're* the caravan under attack and your towers walk." A short-session, stream-friendly, movement-driven escort-strategy game where every road is a gamble between the safe route and the rich one. Nothing in the immediate indie space combines *mobile-base-under-assault* with *pre-mission risk/reward routing* at this session length.

## Recommendation
**Continue development — but redesign the depth systems, don't just add content.**

Carriage Run has cleared the hardest bar: it's a working, polished, distinctive MVP that a real playtester said felt good, and it has already proven it can iterate on feedback (the balance/feel/UX round is done and landed well). It does not need reduced scope or archiving. What it needs is to spend its next effort on **depth and retention** rather than more missions:

1. Fix the core feel (Roam stance + enemy verbs) — cheap, high-impact, directly addresses tester friction.
2. Deepen progression into decisions (chassis + choice upgrades) — the tester's stated milestone.
3. Add a roguelite meta-loop — the missing retention ingredient.

Adding a 13th mission to a loop that fatigues and doesn't retain would be the wrong move. Deepening the loop so the existing 12 missions are worth replaying is the path to a polished, playable, *returnable* experience.
