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
Herdr-inspired keyboard navigation, discrete borders, clear focused panels,
connections/explorer on the left, SQL editor and results on the right.
This structure and implementation plan were approved by the user.
