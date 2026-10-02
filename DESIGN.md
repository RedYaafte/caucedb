---
name: CauceDB
description: Minimal, keyboard-first Oracle workbench for the terminal.
colors:
  background: "#1b1e1f"
  surface: "#25292a"
  selected: "#35312d"
  foreground: "#c8c0ae"
  foreground_bright: "#f1e9d8"
  muted: "#a99f90"
  accent: "#e2a35f"
  border: "#77746c"
  border_soft: "#555650"
  error: "#f18472"
components:
  workspace:
    backgroundColor: "{colors.background}"
    textColor: "{colors.foreground}"
  selected-item:
    backgroundColor: "{colors.selected}"
    textColor: "{colors.foreground_bright}"
  focused-item:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.background}"
  active-tab:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.background}"
  result-grid:
    borderColor: "{colors.border}"
  key-strip:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.foreground}"
---

# Design System: CauceDB

## Direction

The approved [Reference / Workspace proposal](docs/proposals/minimal-reference/01-reference-workspace.png)
replaces the earlier four-box, terminal-palette presentation. The application
keeps four logical focus targets (Connections, Explorer, SQL editor, Results),
but the wide layout has one quiet sidebar and one shared editor/results frame.
The UI is terminal-native: cells and the terminal's monospace font establish
geometry. No desktop window controls, wallpaper, web CSS or animation are used.

This is an **Operate** surface. Color directs attention to the active SQL tab,
focused pane and actionable shortcuts; it does not decorate every label.
The fixed RGB palette is intentional, so appearance is predictable even when
the terminal's ANSI palette differs. The user chose this visual direction after
reviewing both terminal-inherited and Black Ember proposals, then approved the
Black Ember gold accent in a preview of the actual TUI.

## Color roles and contrast

| Role | Value | Use |
| --- | --- | --- |
| Canvas | `#1b1e1f` | Main terminal surface |
| Surface | `#25292a` | Result headers and key strip |
| Selected | `#35312d` | Current connection, object and result row |
| Body | `#c8c0ae` | SQL, data and ordinary copy |
| Bright | `#f1e9d8` | Selected text and major labels |
| Muted | `#a99f90` | Supporting labels and metadata |
| Accent | `#e2a35f` | Active tab, focus and SQL commands |
| Grid | `#77746c` | Row and column rules |
| Soft rule | `#555650` | Structural frame and dividers |
| Error | `#f18472` | Error status with an explicit prefix |

Body text is approximately 9.27:1 against the canvas; muted text is 6.43:1;
grid rules are 3.59:1. The active tab pairs dark text with the gold fill at
approximately 7.70:1. Exact results depend on terminal rendering.

## Layout and behavior

At 90 × 24 cells and above, the sidebar is about one fifth of the terminal,
clamped to 24–30 columns. It shows connections, the current schema's objects
and, when height permits, open SQL files. The right workspace has one border,
document tabs, an editor region and a results region separated by one rule.
The editor takes about 48% of the space below the tabs. A single status row and
contextual key strip follow the workspace.

Below 90 × 24, the existing one-panel-at-a-time mode remains. Tab and
Shift+Tab cycle the four logical targets. Below 45 × 14, the resize notice
replaces the workbench. Modals remain centered and resize with the terminal.

The sidebar deliberately omits per-object boxes and repeated type subtitles.
The saved connection and current object have selected rows. A selected row is
gold with dark text only while its section receives keyboard focus; otherwise
it stays neutral. Section titles follow the same focus rule. The Files list
mirrors the active SQL document and uses the gold focused row when the editor
has focus; it is not a separate Tab target. Long lists, including Files, follow
selection so the active item remains visible. The open documents appear in
tabs; the sidebar repeats their names only when enough height remains.

## SQL and results

The active document tab is gold with dark text. `*` marks unsaved content.
The editor uses warm body text, muted line numbers, a gold cursor and SQL
keyword highlighting. The focused SQL region is named and shows F5, F6 and
Ctrl+S in its heading. Selection and cursor behavior remain those of
`tui-textarea`.

Results use an explicit box-drawing grid: every visible column has a vertical
rule, and every visible row has a horizontal rule. Headers sit on the raised
surface, the selected row uses the selected surface, and the row/column position
is also written in text. Column widths derive from the heading and a small
sample of rows, capped so multiple columns can fit. Arrow-key column movement
shifts the horizontal viewport; the selected row remains visible vertically.
Cell content is sanitized to one line and clipped visually; Enter opens its
full detail. `[ / ]` switches retained results.

## Forms, status and keys

The connection form retains Details, Advanced, User info and Proxy User with
its existing keyboard flow. Focused fields and tabs use gold; passwords stay
masked. Test and connection progress, errors and results are always written as
text, not conveyed by color alone. F1 opens help; the bottom strip names F1
and the transaction keys while no modal is open. F10 still opens message and
execution history.

## Boundaries

- Preserve terminal-owned typography and compact navigation.
- Do not reintroduce separate boxed cards for each workbench section.
- Keep table rules legible; do not use the soft structural border for data.
- Keep essential state in labels or markers as well as color.
- Generated screenshots use synthetic data only.
