# CauceDB

<!-- impeccable:product-schema 1 -->

## Platform
Terminal TUI, Linux first (approved); portable Rust where dependencies allow.

## Stack
Approved: Rust, Ratatui/Crossterm, oracle/ODPI-C, dedicated database workers,
TOML profiles and operating-system keyring. Oracle Instant Client required.

## Product Purpose
Connect to Oracle, inspect database objects, edit and execute SQL files without
leaving the terminal. The adapter boundary must allow future database engines.

## Capabilities and Constraints
Connection profiles with Details/Advanced and User info/Proxy User sections,
test/connect/save actions, optional secure password persistence, SQL/PLSQL
execution, results and explicit transactions. Docker is authorized for testing.
Full SQL*Plus command emulation is outside the MVP.

## Brand Commitments
Keyboard-first navigation. The user selected Reference / Workspace: a minimal
charcoal terminal surface with a restrained Black Ember gold accent, one sidebar
for connections and objects, and one shared SQL editor/results workspace. Grid rules
must make result rows and columns identifiable. Narrow terminals still expose
the four logical focus targets through Tab and Shift+Tab.
