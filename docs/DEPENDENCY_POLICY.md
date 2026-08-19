# Dependency and toolkit reproducibility

toolkit.lock is the single revision source for the shared macroquad-toolkit.
The local Cargo path dependency remains intentional so the game and toolkit
are tested together. CI checks out the exact revision in that file, and
assets/data/game_config.json embeds the same revision in the runtime build
manifest shown to release tooling.

When upgrading the toolkit:

1. Update toolkit.lock to the tested toolkit commit.
2. Update the pinned checkout in .github/workflows/rust-ci.yml and the game
   configuration value in the same change.
3. Run cargo fmt -- --check, cargo clippy --all-targets -- -D warnings,
   cargo test --all-features, and the project publisher.

Crates remain pinned by Cargo.lock; new dependencies require a clear runtime,
build, or release-tooling benefit and must be included in the generated
license inventory.
