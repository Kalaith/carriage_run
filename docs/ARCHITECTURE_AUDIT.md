# Architecture audit

Date: 2026-08-21

## Prioritized findings

### High impact — mission construction mixed with simulation

`state/mission.rs` contained route-choice resolution, accessibility tuning,
guard/equipment setup, seeded randomness, and the live simulation model in one
module. That made balance changes and setup changes harder to review separately.

The setup policy now lives in `src/state/mission/setup.rs`. `MissionRun` keeps
the authoritative route state and delegates fresh-run resolution to
`MissionSetup`.

The same model also owned four unrelated transient feedback fields. Those are
now grouped in `src/state/mission/effects.rs` as `MissionEffects`, keeping
hit-stop, particles, floating text, and screen shake under one explicit owner.

### Medium impact — expedition file mixed persistent and session concerns

`state/journey.rs` combined saved run history, branch/reward value types,
procedural offer generation, and session transitions. The saved history now
lives in `state/journey/records.rs`; branch and reward types live in
`state/journey/options.rs`. The parent module owns the active-run state and
transition rules.

### Medium impact — malformed stake identifiers could rely on an invariant

Expedition startup used an `expect` after a separate affordability check. The
check was normally sufficient, but an invalid selected stake should be a
normal failed action, not a panic boundary. Startup now resolves the stake with
an explicit `Option` path and leaves the campaign unchanged on failure.

### High impact — application shell mixed input and action families

The game shell previously assembled keyboard, touch, and gamepad input in
`game.rs`, while `game/actions.rs` handled navigation, preferences, campaign
progression, expedition flow, and persistence in one match. The shell now has
explicit boundaries: `game/input.rs` owns device-to-intent translation;
`action_navigation.rs`, `action_settings.rs`, and `action_expedition.rs` own
their corresponding action families; and `actions.rs` retains the central
audio hook plus campaign/progression and save-slot dispatch. Gameplay behavior
and action ordering are unchanged.

## Deliberately deferred debt

- `Game` remains the application shell for startup, rendering, audio,
  autosave, notifications, and persistence. The action dispatcher is now split
  by family, but campaign/progression and save-slot handling still share
  `game/actions.rs`. The next safe seam is to separate those remaining action
  families or extract startup/render services if new features increase their
  size.
- `state.rs` still contains the serialized `CampaignState` aggregate and much
  of its normalization logic. The aggregate is intentionally a single save
  boundary; its normalization helpers are a future extraction candidate if
  save migration work expands.
- Several UI modules are close to the 800-line project limit:
  `gameplay_hud.rs` (798), `management.rs` (782), `upgrade_visuals.rs` (744),
  and `mission_map.rs` (727). They should be split before the next substantial
  feature is added to each screen.

## Verification baseline

The repository already routes embedded JSON through `macroquad-toolkit` and has
headless mission, progression, persistence, balance, and content-validation
tests. The refactor adds coverage for invalid expedition stake state while
preserving the existing gameplay paths.
