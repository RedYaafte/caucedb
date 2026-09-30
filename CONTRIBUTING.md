# Contributing to CauceDB

Issues and pull requests are welcome. The 0.1 preview is Oracle-only; please
describe proposed support for other engines before implementing a large change.

Run these checks before a pull request:

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

The Oracle integration tests are ignored by default. Use only an isolated,
disposable Oracle instance for them; setup is documented in the README. Never
attach CI or test scripts to a shared DEV, QA or production database.

Please include a concise description, tests for changed behavior and any
documentation updates. Do not submit credentials, real connection profiles,
Wallets, database exports, customer data or screenshots containing private
database information. The `.gitignore` excludes common local artifacts, but
review `git diff --cached` before committing.

The project uses the MIT license. By submitting a contribution you agree that
it can be distributed under that license. Oracle Instant Client is a separate
third-party dependency and must not be added to the repository or release
archives without a deliberate review of Oracle's license terms.
