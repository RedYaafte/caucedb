# Black Ember / Classic hybrid proposals

These previews combine the earlier CauceDB workbench with the [Black Ember palette](https://github.com/RedYaafte/black-ember/blob/main/palette/black-ember.toml). They preserve the earlier two-left-pane layout, editor/results split, integrated border titles, status line, and keyboard footer. Data is synthetic. These are historical proposals, not screenshots of the implemented UI.

| Proposal | Current TUI element retained | Black Ember treatment |
| --- | --- | --- |
| [Terminal Classic](01-terminal-classic.png) | Sparse pane styling and full-row selection | Canonical warm text, amber focus border, and readable grid lines |
| [Amber Cursor](02-amber-cursor.png) | Same layout and pane navigation | Layered row shading and a clearly filled active cell |
| [Quiet Grid](03-quiet-grid.png) | Spacious SQL editor and compact explorer | Deeper background, restrained accents, and outlined active cell |

All three use Black Ember's `#e8dfd0` foreground, `#a69a8b` muted text, and `#8b8176` result-grid rules. The grid contrast against the standard `#171614` canvas is approximately 4.74:1. The third proposal uses the canonical `#11100f` deep background, raising that contrast further. Horizontal and vertical rules identify both row and column boundaries; the selected row or cell also has a non-color cue through fill or outline.

Historical recommendation: **Amber Cursor** balanced that earlier TUI with a
precise location indicator for wide result sets. The selected direction is now
[Reference / Workspace](../minimal-reference/01-reference-workspace.png), with
the [gold-accent implementation](../../screenshots/reference-workspace.png).

Run `node docs/proposals/classic-hybrids/render.mjs` to regenerate the SVG mockups. PNGs are rasterized copies for review.
