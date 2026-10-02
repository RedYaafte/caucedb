# Color proposals for CauceDB

These are static design previews for review, not screenshots of implemented UI. All query results and connection details are synthetic. The first two directions use [Black Ember's canonical palette](https://github.com/RedYaafte/black-ember/blob/main/palette/black-ember.toml) as their base; the third is a new palette inspired by it.

| Direction | Approach | Selection | Result grid |
| --- | --- | --- | --- |
| [Black Ember / Classic](01-black-ember-classic.png) | Original warm graphite and amber roles | Full row | Warm, explicit row and column rules |
| [Black Ember / Precision](02-black-ember-precision.png) | Same base palette, quieter surface layers | Current cell plus row guide | Explicit rules and alternating row surfaces |
| [Cauce Ember](03-cauce-ember.png) | Sage-shifted graphite with warm amber focus | Current cell plus row guide | Higher-contrast sage rules and alternating rows |

All directions use text and shape, not color alone, to convey selection. The line colors contrast against their canvas by approximately 4.74:1 for Black Ember and 3.66:1 for Cauce Ember. The primary text contrast is approximately 13.69:1 and 14.17:1, respectively. Those figures apply to the proposed hex values, not necessarily to a terminal emulator's interpretation.

Historical recommendation: **Black Ember / Precision**. These previews informed
later exploration; the selected Reference / Workspace layout and gold accent are
now implemented. See the [current Ratatui render](../screenshots/reference-workspace.png).

Regenerate the SVG files with `node docs/proposals/render.mjs`. The PNG files are rasterized copies of the SVG previews.
