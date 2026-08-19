# Font fallback assets

The two runtime fallback files are copies of the shared
macroquad-toolkit Rajdhani SemiBold font. They are kept under the game asset
tree so the browser build can resolve them without depending on a host OS
font. The toolkit's font license travels with the shared toolkit repository
and is included in the third-party review.

German and French use latin_extended.ttf first, then english.ttf. The
localization audit checks long strings before release.
