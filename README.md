# CauceDB

CauceDB is a keyboard-first database workbench for the terminal, written in Rust.
This **0.1 preview supports Oracle only**. PostgreSQL, MySQL and SQLite are
future goals, not currently supported backends. The interface uses the terminal's
own color palette and keeps connections, object browsing, SQL files and results
in one workspace.

This is an early preview intended for development and QA evaluation. Review SQL
before execution and use least-privilege accounts. Production use has not been
validated.

## Linux installation

Install the current Linux x86-64 preview to `~/.local/bin` with one command:

```sh
curl -fsSL https://raw.githubusercontent.com/RedYaafte/caucedb/main/install.sh | sh
```

The installer downloads the `v0.1.0-alpha.1` release, verifies its SHA-256
checksum, and installs atomically without `sudo`. It requires `curl`, `tar`,
`sha256sum`, `awk`, `mktemp` and standard Linux tools. Set
`CAUCEDB_INSTALL_DIR=/absolute/path` to choose another writable directory, or
`CAUCEDB_VERSION=v0.1.0-alpha.1` to select a published version. The installer
will tell you if its directory is not in `PATH`. For review before execution,
[read the installer source](install.sh) instead of piping it directly to `sh`.
It does not install Oracle Instant Client or change your shell configuration.

For example, to install in a different directory:

```sh
curl -fsSL https://raw.githubusercontent.com/RedYaafte/caucedb/main/install.sh | CAUCEDB_INSTALL_DIR="$HOME/bin" sh
```

For a manual installation, download the `x86_64-unknown-linux-gnu` archive and
its `.sha256` file from
[GitHub Releases](https://github.com/RedYaafte/caucedb/releases), then verify and
extract them:

```sh
sha256sum -c caucedb-v0.1.0-alpha.1-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -xzf caucedb-v0.1.0-alpha.1-x86_64-unknown-linux-gnu.tar.gz
./caucedb --help
```

The example filename is for the first preview tag. Use the filenames of the
release you downloaded. Releases contain the CauceDB executable, README and
MIT license, but **not** Oracle client libraries. You
must install [Oracle Instant Client Basic or Basic Light](https://www.oracle.com/database/technologies/instant-client/downloads.html)
separately on each machine, with the same CPU architecture as CauceDB. Linux
also needs the runtime libraries required by your Instant Client version,
including `libaio` where applicable. See Oracle's installation instructions for
your distribution. No Rust toolchain is needed to run a downloaded binary.

```sh
export LD_LIBRARY_PATH=/absolute/path/to/instantclient
./caucedb
# Or open a SQL file on startup:
./caucedb /absolute/path/to/query.sql
```

The Linux binary is built on Ubuntu 22.04; use a compatible glibc-based Linux
distribution. Other CPU architectures and Linux compatibility targets are not
release-tested yet. If the Oracle client is missing, the UI still opens and
reports the loading problem when you connect. Oracle Wallet, `tnsnames.ora` and
`sqlnet.ora` are your own local files and are not included in releases.

To build from source instead, install a recent stable Rust toolchain and a C
compiler, then run `cargo build --release --locked`. The executable will be at
`target/release/caucedb`. Install Instant Client before connecting. macOS and
Windows keyring backends are configured in the code but not validated releases.

## Connect to DEV or QA

Start `caucedb` and press `F2` to create or edit a connection. Fill in Connection
Name, keep Connection Type as Oracle, and provide Hostname, Port and Service Name
or SID. Under User info, choose authentication type and role, then enter the
username and password. `F5` tests a temporary authenticated connection; `F6`
connects; `F2` saves the profile without connecting. A successful Test performs
a ping and `SELECT 1 FROM dual`.

For Oracle Net aliases or a full descriptor, select **TNS / Descriptor** in the
connection form. Set `TNS_ADMIN` before launching CauceDB if Oracle Net files are
needed. Advanced settings include TCP/TCPS, Wallet location, connection and
per-call timeouts, and a row limit. TCPS enables server name matching. External
authentication and SYSDBA/SYSOPER require appropriate database-side
configuration and are not covered by the basic live integration test
environment. Proxy authentication is covered by the disposable Oracle test.
See [connection details](docs/connections.md).

Use separate profiles and least-privilege Oracle accounts for DEV and QA. The
application currently has **one active database session** shared by every SQL
tab. Autocommit is off. `F7` commits and `F9` rolls back. Changing connections
or quitting prompts if work may be pending; disconnect and normal shutdown roll
back. Oracle DDL may commit implicitly. There is no read-only mode that prevents
all writes, so avoid connecting with privileged accounts just to inspect data.

## Keyboard reference

| Action | Key |
| --- | --- |
| Switch panels | Tab / Shift+Tab |
| Help | F1 |
| Edit connection | F2 |
| Select explorer schema | F3 |
| Save as | F4 |
| Run selection or statement at cursor | F5 |
| Run entire file | F6 |
| Commit / Rollback | F7 / F9 |
| Cancel query or script | F8 |
| Message and error history | F10 |
| New / open / save SQL file | Ctrl+N / Ctrl+O / Ctrl+S |
| Close SQL file | Ctrl+W |
| Switch SQL files | Ctrl+Left / Ctrl+Right |
| Search regular expression | Ctrl+F |
| Select text | Shift+arrow keys |
| Quit | Ctrl+Q |

In **Connections**, `n` creates, `e` edits, Enter connects and `d` disconnects.
In **Explorer**, `/` filters, `r` reloads, Enter/`1` shows columns, `2` keys,
`3` indexes, `4` PL/SQL source and `p` opens a table preview. In **Results**,
arrows navigate, PgUp/PgDn scroll, Enter opens a cell, and `[`/`]` switch result
sets. Help and details scroll with arrows.

At 90 × 24 terminal cells or larger, all four panels are shown. Smaller
terminals show the focused panel; the minimum supported size is 45 × 14. Some
terminal emulators reserve function keys or Ctrl+S; adjust terminal bindings if
needed.

## SQL files and results

The editor opens, creates and saves UTF-8 `.sql` files, with tabs, line numbers,
basic SQL highlighting, selection, search and undo/redo. PL/SQL blocks end with
`/` on a line of its own:

```sql
SELECT user, sysdate FROM dual;
BEGIN
    NULL;
END;
/
```

Script execution stops at the first error and retains preceding results. The
splitter handles comments, escaped quotes, quoted identifiers and Oracle
`q'[...]'` literals. SQL*Plus commands (`SET`, `SPOOL`, `@`, `&` variables) are
not emulated. UI bind parameters, DBMS_OUTPUT, PL/SQL debugging, SSH tunnels,
binary export and non-Oracle engines are not available yet.

Preview limits: 8 MiB per file, 1,000 statements per run, last 20 result sets,
1,000 rows by default (configurable 1–100,000), 8 MiB of accumulated text per
result and 4,096 characters per cell. CLOB/NCLOB show a 4,096-byte preview;
BLOB shows 128 bytes in hexadecimal. Unsupported data types are marked. These
limits are indicated in the UI when reached. `F8` asks the driver to cancel the
active execution and stops the remaining script; it cannot guarantee an
immediate interruption of a connection attempt.

## Profiles, passwords and troubleshooting

On Linux, profiles are stored at `~/.config/caucedb/connections.toml` unless
XDG configuration paths differ. Set `CAUCEDB_CONFIG` to use a specific file.
Passwords are never written to that TOML file. **Save password** uses the
operating system keyring (Secret Service on Linux). If the keyring is unavailable,
such as in some SSH sessions, leave that option unchecked and enter the password
each time. CauceDB does not fall back to plaintext secret storage. Keep Oracle
Wallets outside the repository and protect them with filesystem permissions.

Optional diagnostics: set `CAUCEDB_LOG=/path/to/caucedb.log` and `RUST_LOG=info`.
The log records operation categories and failures, not SQL, result values,
descriptors or credentials. The in-app message history (`F10`) may contain
database error details, so inspect it before sharing screenshots or reports.

If you used an earlier local `oracle-tui` build, its configuration directory and
keyring service name differ. Recreate profiles in CauceDB or copy the old TOML
file into the new config location, then re-enter any saved passwords.

## Development and tests

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

The four Oracle integration tests are ignored by default. For an isolated
Oracle Free container, set a **disposable** alphanumeric password and install
Instant Client on the host. This container is bound to localhost only.

```sh
export ORACLE_TEST_PASSWORD='choose_a_disposable_password'
docker compose up -d --wait
docker compose exec -T oracle sqlplus -s / as sysdba @/dev/stdin "$ORACLE_TEST_PASSWORD" < dev/proxy.sql
export LD_LIBRARY_PATH=/absolute/path/to/instantclient
cargo test --test oracle_integration -- --ignored --test-threads=1
docker compose down
```

The proxy setup command passes the disposable password as a process argument;
do not use a real credential there. `dev/proxy.sql` creates `tui_proxy` once; for
repeat runs against the same container, skip that step or reset the container
intentionally. The tests create uniquely named tables under `TUI_TEST`, exercise
querying, metadata, transactions, PL/SQL, proxy auth and cancellation, then
drop their own tables. Do not point these tests at a shared DEV or QA database.

Visual inspection without Oracle: `cargo run -- --snapshot`,
`cargo run -- --snapshot form`, or `cargo run --example render`. The example
generates SVG snapshots using synthetic data in `.local/screens/`.

## Project status and license

This release supports one Oracle session. The database trait is a starting
point, not a finished multi-engine plugin system. See [the roadmap](docs/roadmap.md)
and [contributing guide](CONTRIBUTING.md). CauceDB is licensed under [MIT](LICENSE).
Oracle Instant Client is third-party software distributed under Oracle's own
[license terms](https://www.oracle.com/downloads/licenses/instant-client-lic.html)
and is **not bundled** with CauceDB.
