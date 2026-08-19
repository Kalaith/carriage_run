# Carriage Run

Carriage Run is a Rust + Macroquad escort strategy game. Steer a supply
carriage across dangerous roads while hired guards protect the cargo, rescue
prisoners, and carry the campaign into its final acts.

## What's built

- A 30-mission campaign spanning three acts, three authored biomes, route
  branches, six side missions, two campaign bosses, and a designed finale.
- An 8-leg expedition mode with branching legs, modifiers, relics, events,
  entry stakes, seeded/daily runs, persistent tokens, records, and a finale
  boss.
- Carriage chassis, frame tunings, equipment, damage states, bobbing wheels,
  guard and enemy animation, biome road art, particles, hit-stop, and screen
  shake.
- Swordsman and Mage 3-star specializations, guard barks and hire quotes,
  cosmetic livery and guard-color unlocks, and roster flavor text.
- Breakout attempts, Rockslide, Cursed Fog, and Night Stretch hazards with
  authored balance coverage and counterplay feedback.
- Touch-first route controls, visible tutorial guidance, controller actions,
  keyboard rebinding, threat/off-screen indicators, tooltips, confirmations,
  results explanations, credits, and screen transitions.
- Settings for display, audio, text size, colorblind palette, reduced motion,
  drag behavior, language, and recovery-safe multi-slot saves with rolling
  backups and corruption quarantine.
- English, German, and French localization with fallback diagnostics, bundled
  font fallbacks, layout checks, generated audio, and page-focus mute support.

## Run

```powershell
cargo run
```

## Validate and package

```powershell
.\publish.ps1
```

The publisher is the project validation path. It builds and packages the
Windows and WebGL targets, checks release budgets, and deploys the result to
the local WebHatchery game directory.

```powershell
.\scripts\browser_smoke.ps1
```

Release budgets, dependency/version policy, license inventories, and the
latest verification captures live under `docs/`.

## Project status

The release backlog in [TODO.md](TODO.md) is complete. Remaining work should
be tracked as new issues or milestones rather than carried as unfinished
prototype scope.
