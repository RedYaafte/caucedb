# Minimal TUI proposals

These are static CauceDB design previews, not screenshots of the running application. All connection names, schema objects, and query results are synthetic. The first two proposals take their visual language only from the user-provided reference image; the third combines that image's shell and spacing with [Black Ember's canonical palette](https://github.com/RedYaafte/black-ember/blob/main/palette/black-ember.toml) and the earlier [Precision proposal](../02-black-ember-precision.png).

**Selected and implemented:** Reference / Workspace. See the [current Ratatui render](../../screenshots/reference-workspace.png). The implemented screen follows the proposal within terminal-cell constraints and now uses the approved [Black Ember gold accent](../gold-accent/reference-workspace-gold.png). This original orange mockup remains the layout reference.

| Proposal | Reference treatment | Results focus |
| --- | --- | --- |
| [Reference / Workspace](01-reference-workspace.png) | One primary frame, quiet sidebar, orange active file tab | Restrained full-row highlight |
| [Reference / Data First](02-reference-data-first.png) | Compact editor and larger results area | Reference-style orange selected row |
| [Black Ember / Minimal Precision](03-ember-minimal-precision.png) | Minimal shell and sidebar with Black Ember colors | Warm row guide plus amber current cell |

Every proposal keeps connection status, schema browsing, SQL files and tabs, statement/script execution, results navigation, transaction controls, and help visible or discoverable through labeled shortcuts. The result grid has both horizontal and vertical rules. The terminal and desktop window chrome shown in the inspiration image are intentionally not copied into CauceDB.

The image-led palette is approximated from the supplied screenshot: `#1b1e1f` canvas, `#c8c0ae` body text, `#a99f90` secondary text, `#77746c` table rules, and `#ed660c` accent. Body text and table rules contrast with the canvas by approximately 9.27:1 and 3.59:1. The hybrid uses Black Ember's `#171614`, `#e8dfd0`, `#8b8176`, and `#e2a35f` roles instead.

Regenerate SVGs with `node docs/proposals/minimal-reference/render.mjs`; PNGs are rasterized copies for review.
