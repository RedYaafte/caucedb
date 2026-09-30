# Roadmap

This roadmap is directional, not a delivery commitment.

## 0.1 preview: Oracle on Linux

- Connection profiles, authenticated test, Oracle session, object inspection,
  SQL/PLSQL execution, SQL file editing and explicit transactions.
- Linux x86-64 binary releases built on Ubuntu 22.04, with Oracle Instant Client
  installed separately by the user.
- Automated unit/workflow tests; Oracle Docker integration tests run manually.

## Before a stable Oracle release

- Validate installation and usability on multiple real Linux distributions and
  terminal emulators, including DEV and QA workflows.
- Test keyring behavior in desktop and headless sessions; validate advanced
  Oracle modes such as TCPS/Wallet, TNS, proxy and external auth where available.
- Improve large-result navigation, error diagnostics and script edge cases.
- Add release signing or provenance, and an automated Oracle integration test
  environment if its maintenance and licensing are acceptable.

## Multi-engine work

- Decouple the worker and metadata queries from the Oracle implementation.
- Model connection fields and SQL dialect behavior by backend.
- Add another backend only after its connection, query, transaction, explorer
  and file-execution behavior has integration tests. SQLite is a candidate for
  the first additional engine; PostgreSQL and MySQL are planned, not promised.
