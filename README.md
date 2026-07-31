# Carriage Run

Carriage Run is a Rust + Macroquad escort strategy prototype based on the supplied game design document. The player steers a supply carriage through scrolling routes while hired melee and ranged guards protect it from wolves, bandits, and skeletons.

## What's built

- 12-mission campaign with route branches, including prisoner, princess, monster-egg, and
  siege escort variants — each with its mission-type meter promoted to a real verb.
- Expedition mode as a full roguelite: an 8-leg arc with FTL-style branching legs and
  modifier twists, choice-of-3 leg rewards, run-scoped relics, between-legs run events,
  entry stakes, seeded/daily runs, persistent tokens unlocking starting relics, and a
  records screen.
- Three purchasable chassis plus four mutually-exclusive carriage frame tunings.
- Difficulty presets and three assist toggles; a help/bestiary codex screen.
- Mission loadout screen with multiple melee slots and carriage ranged slots.
- Shop screen for hiring permanent melee and ranged guard recruits.
- Dedicated guard roster screen with guard star upgrades and unique 2-star/3-star ability text.
- Settings screen with route motion, alert, and autosave toggles.
- Pause menu during routes with resume, settings, save, and route-map options.
- Carriage equipment slots for active armor, wheel, cargo, and repair systems.
- Visual carriage changes for chassis level and equipped systems.
- Carriage steering with weighted horizontal movement.
- Drag orders for guards: attack enemies, move to ground, return to escort formation, or mount/dismount ranged guards on carriage slots.
- Ranged guard choices: Archer, Crossbow Guard, and Mage.
- Guard injury recovery after defeat.
- Wolves, bandits, bandit archers, skeletons, and necromancers that can raise skeletons.
- Mud, fallen trees, rocks, and fire patches.
- Mission-specific security and comfort meters.
- Mission scoring, star ratings, gold rewards, and campaign records.
- Upgrade screen for carriage armor, guard training, mounted ranged slots, wheels, cargo straps, and repair kits.
- Emergency repair kit upgrade with a once-per-route repair ability.
- Melee guard choices: Swordsman, Shield Guard, and Spearman.
- Toolkit-backed save, load, notifications, virtual UI, asset pack hook, and WebGL publishing path.

## Run

```powershell
cargo run
```

## Validate

```powershell
.\publish.ps1
```

The workspace standard validation path is the project publisher, which builds and packages the Windows and WebGL targets.

```powershell
.\scripts\capture_ui.ps1 -Scenes map,loadout,journey   # headless UI screenshots
```

## Open work

See `TODO.md` — the remaining gaps toward a commercial release, grouped by area.
