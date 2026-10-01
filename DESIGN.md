---
name: CauceDB
description: Herdr-inspired keyboard workbench for Oracle sessions and SQL files.
colors:
  background: "terminal-default-background"
  panel: "ansi-dark-gray"
  foreground: "terminal-default-foreground"
  muted: "ansi-dark-gray"
  accent: "ansi-cyan"
  amber: "ansi-yellow"
  border: "ansi-dark-gray"
  error: "ansi-red"
components:
  panel:
    backgroundColor: "{colors.background}"
    textColor: "{colors.foreground}"
  selected-item:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.accent}"
  active-field:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.foreground}"
  editor-cursor:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.background}"
  key-strip:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.accent}"
---

# Design System: CauceDB

## Overview

**Creative North Star: "Herdr-inspired Oracle workbench"**

A dense, keyboard-operated terminal workspace that inherits the terminal's
active OS theme. Default terminal surfaces and text, ANSI focus and yellow
session state organize connections,
objects, editable SQL and results. The approved visual direction is recorded
in PRODUCT.md and docs/surface.md.

This document describes the implemented Ratatui interface in src/ui.rs and
src/app.rs. Terminal cells determine geometry; the terminal owns the font.
No browser styling, raster assets or animation are part of the interface.

**Key Characteristics:**

- Four numbered panels with contextual keyboard guidance.
- ANSI cyan focus, selected rows and active tabs against the terminal canvas.
- Adaptive single-panel presentation and compact, scrolling connection forms.
- Persistent session, execution and transaction feedback.

## Colors

The palette is delegated to the terminal. `Reset` inherits the user's default
foreground/background, while ANSI named colors resolve through the terminal's
configured OS theme. No RGB values are embedded in the UI.

### Primary

- **ANSI cyan (`accent`):** focused borders and titles, selected list entries,
  active tabs, result headers, keyboard actions and recognized SQL keywords.
  The editor cursor reverses foreground and background using this accent.

### Secondary

- **ANSI yellow (`amber`):** the top-line running timer, pending
  transaction notice and autocommit-off state.
- **ANSI red (`error`):** status text when an operation reports an error;
  the same line also begins with `ERROR ·`.

### Neutral

- **Terminal default (`background`):** canvas, panel bodies and inactive form
  fields, inherited from the active terminal theme.
- **ANSI dark gray (`panel`):** selected rows, cursor line, focused form values,
  active tabs, table headers and keyboard strips.
- **Terminal default (`foreground`):** ordinary content and input values.
- **ANSI dark gray (`muted`):** inactive titles, labels, object types, line
  numbers, empty-state guidance and ordinary status messages.
- **ANSI dark gray (`border`):** unfocused panel outlines.

The exact appearance changes with the user's terminal theme, including light
and dark OS schemes. ANSI colors are semantic roles, not fixed visual values.

**The Focus Rule.** Focus changes the panel border and title together. Selection
inside a panel also uses a tonal background; ANSI cyan alone does not identify which
panel receives keys.

## Typography

The terminal supplies one monospace font and one cell size. There are no bundled
fonts or application-controlled font sizes, line heights or tracking values.

- **Identity:** `CAUCEDB /` uses bold ANSI cyan on the session line.
- **Panel titles:** numbered, single-line labels embedded in borders; ANSI cyan when
  focused and muted otherwise.
- **Body and values:** ordinary pale text; labels and supporting copy are muted.
- **Editor:** muted line numbers, a highlighted cursor line and ANSI cyan SQL keywords.
  Indentation uses a tab length of four. Syntax coloring preserves editor geometry
  and the cursor background.
- **State markers:** `›` marks list selection, `●` marks the connected profile,
  and `*` marks a modified document. These markers supplement color.

## Layout

All dimensions below are terminal columns or rows, never CSS pixels.

The shell reserves two rows for session identity, a flexible work area, two rows
for status and one bottom row for contextual keys. At widths of at least 90 and
heights of at least 24, the work area shows all four panels. The left column is
one quarter of the terminal width, clamped to 24–34 columns; Connections takes
eight rows and Explorer fills the remainder. On the right, SQL editor and Results
split the available height 52%/48%. Document tabs occupy one row above the editor.

Below either full-layout threshold, only the focused panel occupies the work
area. Tab and Shift+Tab continue cycling through all four panels. Below 45 columns
or 14 rows, a resize notice replaces the workbench and names Ctrl+Q and Esc.

Dialogs are centered within the terminal area above the three status/help rows,
with at least one cell of surrounding margin. Their requested width is 90;
requested heights are 28 for the connection form and help, 26 for cell detail,
and 16 for prompts and confirmations. Dimensions shrink to the available area.

The connection form becomes compact when its inner width is below 80 or its
inner height below 22. Each field occupies two rows. Labels use up to 29 columns,
capped at half the field width; values fill the remainder. The visible field
window follows selection. Compact mode reduces tab and hint space while keeping
the two-row action area. Long input values show their trailing characters using
terminal display width.

## Elevation & Depth

Depth is flat and tonal. There are no shadows, gradients or animated transitions.
Dialogs clear and redraw their rectangle over the workbench, using the same
terminal-default background and an ANSI accented border. ANSI dark gray marks active content within
the deeper canvas.

## Shapes

Panels and dialogs use single-line rectangular terminal borders with square
corners. Titles have one space on either side. Dividers, border characters and
aligned cells supply structure; there are no rounded cards or graphical buttons.
Boolean fields display `[x]` or `[ ]`, option fields append `‹ ›`, and focused
text fields append `▏`. Passwords display a bullet per character.

## Components

### Workbench panels and lists

Connections and Explorer use stateful lists with a `› ` selection prefix and
ANSI cyan text on the ANSI dark gray selection background. Explorer objects occupy two
lines: name, then muted lowercase type. Empty panels state the next available
action, including `n` for a new connection and connection guidance for Explorer.

### SQL documents

Document tabs show an ordinal, filename (or Untitled.sql) and dirty marker. The
active tab uses ANSI cyan on ANSI dark gray; tabs have vertical-line separators.
Ctrl+Left/Right changes documents. The editor receives its distinctive cursor
only when focused with no dialog open. F5 executes a statement or selection;
F6 executes the file. Ctrl+S saves; Ctrl+O opens a file.

### Results table

Headers use ANSI cyan on ANSI dark gray; selected rows use ANSI dark gray. Column spacing
is two cells, with a minimum width of 16 per visible column. The viewport chooses
at least one column from the available width, starting at the selected column.
The bottom border reports row and column position and `LIMIT REACHED` when
applicable. Multiline cell content becomes a single table line; Enter opens
wrapped, scrollable detail. Arrow keys navigate, PgUp/PgDn move 20 rows, and
brackets switch result sets. Empty results explain F5 and F6.

### Connection form

The four section tabs are Details, Advanced, User info and Proxy User. Connection
Name and Connection Type belong to every section's field list. Details contains
host, port, address type, service/SID and protocol. Advanced contains descriptor,
wallet, timeouts and row limit. User info contains authentication, role, target
username, password and password persistence. Proxy User contains its enable
toggle and credentials.

PgUp/PgDn changes sections; Tab/Shift+Tab or Up/Down cycles fields. Left/Right,
Space or Enter changes an option. Ctrl+U clears text, and Backspace removes the
last character. The focused label becomes ANSI cyan and its value receives the
ANSI dark gray background. F5 tests, F6 connects, F2 or Ctrl+S saves, Esc closes
the form and F8 requests cancellation (Oracle connection setup may wait for its
timeout). The action strip and contextual help remain explicit.
While testing or connecting, the form replaces its contextual hint with an
elapsed-time status. Validation errors and test outcomes appear in that same
space; the global status line remains visible below the dialog. Saving closes
the form and names the saved profile in the status line.

### Status, prompts and reference dialogs

The top line names the active connection or Disconnected, followed by running
elapsed time, a pending-transaction notice or AUTOCOMMIT OFF. The lower status
area wraps messages; errors use ANSI red and an explicit prefix. F7 commits and F9
rolls back. Contextual bottom hints change with panel and dialog state.

Text prompts show purpose-specific guidance and Enter/Apply, Esc/Cancel and
Ctrl+U/Clear. Confirmations use `y` to confirm and `n` or Esc to cancel. Help and
detail dialogs wrap and scroll with arrows or page keys; Esc closes them.
F1 opens keyboard help and F10 opens messages, errors and execution history.

## Do's and Don'ts

### Do:

- Do preserve terminal-cell geometry and terminal-owned monospace typography.
- Do change focused borders and titles together and keep keyboard hints contextual.
- Do keep the four panels reachable when the terminal shows only one at a time.
- Do pair meaningful color with labels, position or state markers.
- Do keep the selected form field visible and passwords masked.

### Don't:

- Don't introduce browser components, CSS dimensions or raster assets into this native interface.
- Don't replace the approved bordered workbench with a different visual world.
- Don't hide execution errors or transaction state behind color alone.
- Don't add invented font scales, rounded corners, shadows or motion tokens.
