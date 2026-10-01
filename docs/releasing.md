# Releasing CauceDB

The first supported release target is Linux x86-64 with glibc. GitHub Actions
builds on Ubuntu 22.04 and publishes the CauceDB executable, README, license
and SHA-256 checksum. Oracle Instant Client is not included.

1. Update the version in `Cargo.toml` and regenerate `Cargo.lock`. Keep the
   release tag in sync with the package version (for example,
   `0.1.0-alpha.1` corresponds to `v0.1.0-alpha.1`).
2. Run `cargo fmt --all -- --check`, `cargo test --locked`,
   `cargo clippy --all-targets --locked -- -D warnings` and
   `cargo build --release --locked` locally. Run the ignored Oracle tests against
   an isolated disposable database when Oracle behavior changed.
3. Review the README's manual installation instructions, release filename
   example, known limitations and any security-sensitive files in
   `git diff --cached`. Keep the installer pinned to the last published tag
   until the new release assets are live.
4. Merge to `main` and wait for the CI workflow to pass.
5. Create and push an annotated version tag. The release workflow runs on tags
   beginning with `v`, creates a prerelease and attaches the archive and checksum.
6. Download the release archive on another Linux machine, verify the checksum,
   test `--help`, run `install.sh` against the published release, start the TUI
   and connect to a non-production Oracle database
   with a separately installed Instant Client. Record the distribution, terminal
   emulator and Oracle client version in any compatibility report.
7. Update the default `CAUCEDB_VERSION` in `install.sh` and the README's
   one-line-install version note to the new tag, then merge that follow-up to
   `main`. This avoids pointing new installs at assets that do not exist yet.

The release job uses GitHub's repository token with `contents: write` to create
the release. Repository Actions settings must allow that workflow to run. Never
place real credentials, Wallets or customer data in release assets or logs.
