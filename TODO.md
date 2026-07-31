# TODO — Carriage Run

Open work toward a commercial release (Steam + itch, web build as the free demo).
Everything below is still unbuilt; the campaign loop, expedition roguelite, difficulty
presets, chassis/frames, mission-type verbs, and the feedback round are done.

## Owner decisions still needed

- Art direction: commissioned sprite set vs. asset-pack base vs. deliberate stylized-vector.
- Platform scope: Steam Deck verification? mobile ever? — this gates how hard gamepad work bites.
- Price point and content bar: $10–15 premium (needs the full content expansion) vs. $5–8 compact.
- Localization tier: EFIGS only vs. adding CJK (font and cost implications).

## Game depth

- Cosmetic gold sink (carriage liveries, guard colors) once art exists; optional prestige/NG+.
- Prisoner escort: periodic breakout attempt instead of a passive security meter.
- 3★ guard specialization forks (Swordsman → Berserker/Bodyguard, Mage → Pyromancer/Warden).
- Rebalance upgrades so no build can max every track in one campaign.
- 2–3 multi-phase bosses for the campaign and expedition finales; 3–4 more standard enemies.

## Content volume

- Campaign expansion to ~24–30 missions across 3 acts/biomes with distinct hazard and enemy palettes.
- 2–3 new hazards per biome (rockslides, cursed fog, night stretches) — only River Ford shipped.
- 2–3 new guard classes; 1–2 new chassis plus 3–4 equipment items so 4-slot choices stay contested.
- Designed campaign finale with a boss and an ending screen.
- World framing: name the region and acts, stylized map background tying the missions into one journey.
- Guard barks / hire quotes and per-class flavor in the roster.

## Art and game feel

- Asset production: chassis sprites with damage states, guard and enemy animation sets, 3 biome
  tile/prop sets, UI icon set, screen art; sprite loading and a texture atlas via the toolkit.
- Screen shake and hit-stop; more particle emitters (mud splash, embers, arrow trails, summon FX);
  consider extracting the particle system into `macroquad-toolkit`.
- Tween/easing helpers for UI transitions; telegraph VFX on wave spawns.
- Carriage bob, wheel rotation, horse animation.

## Audio (nothing exists)

- SFX set (~40–60): UI, per-weapon combat, carriage rolling loop pitched by speed, alerts, stingers.
- Music: title, 2–3 gameplay tracks, results/shop ambient, expedition hub, boss.
- Wire the toolkit `SoundManager` onto `Game`, event-driven hooks off the pending-hit collection,
  per-screen music state machine, ducking on pause.
- SFX/music/master sliders in Settings (needs the codebase's first slider widget), mute on web focus loss.

## Onboarding and UX

- Guided first mission: contextual prompts for steer, wave telegraph, drag order, Roam toggle,
  mounting a ranged guard, brake/boost — first campaign only.
- Tooltip system (toolkit candidate) on shop items, upgrades, equipment, guard cards, meters, HUD.
- Confirmation dialogs — New Campaign silently overwrites the autosave today; also chassis purchase
  and expedition abandon.
- Post-mission score breakdown explaining *why*, plus a "what to buy next" nudge on early shop visits.
- Menu transitions, hover-state audit, controller focus states, credits screen.
- In-mission readability: threat indicator, off-screen enemy pips, stance-ring legibility.

## Settings, accessibility, input

- Settings expansion: fullscreen/resolution, vsync/FPS cap, UI scale (`set_ui_text_scale` is unused).
- Accessibility: colorblind-safe palette (hit-flash and On/Off badges are red/green), text size,
  reduced motion, hold-vs-toggle for drag interactions.
- Gamepad support — the riskiest UX item; needs a toolkit gamepad module and a non-mouse scheme
  for drag orders. Prototype early.
- Key rebinding: keys are hardcoded literals and `S`/`L` are unmodified global save/load hotkeys.
- Decide touch ambition — `index.html` claims touch support that does not exist.

## Localization

- String externalization to a keyed table (toolkit candidate) covering `src/` literals and the
  text fields in the data JSON — cost grows with every feature, so do it early.
- Font fallbacks per language tier; German/French expansion pass on fixed-width panels.
- Translation at content freeze: EFIGS + Simplified Chinese + PT-BR.

## Technical robustness

- Corrupt-save recovery with a rolling `.bak` — a corrupt autosave silently breaks Continue today.
- Real save timestamps (`SaveSlot.save_date` is hardcoded `"Unknown"`); multiple slots with delete/rename.
- Move autosave onto the unused `AutoSaveManager` timer instead of ~20 per-action write sites.
- Save/load round-trip and corruption-path tests — the persistence layer has no test coverage.
- Panic hook writing a crash log plus a user-facing dialog; startup data-load failure hard-panics silently.
- Opt-in crash reporting and anonymous balance telemetry for native builds.
- Benchmark native `opt-level=3` against the workspace `"z"`; evaluate a fixed timestep for seeded-run fairness.
- WASM size budget: asset atlas and PNG compression (the title PNG alone is 2.6 MB).
- Pin/tag the `macroquad-toolkit` path dependency; single source of truth for the version
  (duplicated in `Cargo.toml` and `game_config.json`), git tags, release automation in CI.

## Distribution and business

- LICENSE/EULA and third-party license inventory (Rajdhani OFL, macroquad, commissioned assets).
- App icon and Windows version resource; Steamworks integration, achievements, Cloud, Deck pass;
  itch.io butler pipeline; optional code signing.
- Steam page live 6+ months before launch: capsule art, screenshots (gated on the art pass), trailer, copy.
- Demo strategy: gate the web build to act 1 plus a limited expedition, add a wishlist link, ship a Steam demo.
- Press kit and GIF library (extend `scripts/capture_ui.ps1` to record footage deterministically),
  devlogs, streamer outreach; trademark/store-collision check on the name.
- Web shell: the loading spinner hides before the WASM download finishes, leaving a blank canvas;
  add a progress bar, WebGL-failure fallback, orientation notice.

## QA and balance

- External playtest program — the game has had exactly one tester. Rounds at alpha, beta, and RC.
- Headless balance simulation harness over the deterministic seeds, asserting win-rate corridors
  and difficulty monotonicity so JSON tuning stops being playtest-gated.
- Full-clear economy audit against the choice-exclusion upgrade model.
- Platform QA matrix (GPUs/DPIs, Deck, browsers) and a save-compat regression suite per release.
- Extend the screenshot capture harness to the newer screens (settings, results, bestiary).
