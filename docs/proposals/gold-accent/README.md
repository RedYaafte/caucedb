# Gold accent approval

This proof-of-concept recolored the actual Ratatui Reference / Workspace
fixture from orange `#ed660c` to Black Ember gold `#e2a35f`. It used synthetic
data and changed no layout or content. The user approved the result, and the
gold accent is now implemented in the TUI. The current generated screenshot is
[here](../../screenshots/reference-workspace.png).

Run `cargo run --offline --example render` and then
`node docs/proposals/gold-accent/render.mjs` to regenerate the SVG. The PNG is a
rasterized copy of that SVG.
