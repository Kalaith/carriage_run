# Asset license inventory

All files under assets/images/, assets/data/, and
assets/packaging/carriage_run.ico are original Carriage Run project assets
created for WebHatchery and distributed with the game under the repository's
project license. No external stock images, fonts, or music files are bundled.

The runtime audio bed is generated deterministically by src/audio.rs through
the shared toolkit synthesizer, so it has no third-party audio license.

Run scripts/generate_license_inventory.ps1 after adding an asset. It scans the
repository and refreshes the machine-readable evidence in
docs/THIRD_PARTY_LICENSES.md.
