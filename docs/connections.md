# Oracle connection guide

CauceDB 0.1 connects to Oracle through the `oracle` Rust driver and Oracle
Instant Client. Install Instant Client separately on every Linux machine before
connecting. Database network access, credentials and privileges are managed by
your organization; CauceDB does not provision them.

## Connection form

| Section | Fields | Notes |
| --- | --- | --- |
| Details | Connection Name, Connection Type, Hostname, Port, Type, Service Name/SID, Protocol | Oracle is the only engine. Use the service name supplied by your DBA; 1521 is merely a common default. |
| Advanced | TNS alias/descriptor, Wallet directory, connection timeout, call timeout, row limit | TNS/descriptor replaces the basic address fields. Timeout values are in seconds. |
| User info | Authentication Type, Role, Username, Password, Save password | Default authentication and role cover ordinary users. Saved passwords require a working OS keyring. |
| Proxy User | Enable, Proxy Username, Proxy Password | The User info username is the target user. Oracle authenticates `proxy[target]`. |

`F5` tests the unsaved form without changing the active connection. It creates a
temporary session, pings it and runs `SELECT 1 FROM dual`. `F6` connects using
the form. `F2` saves the profile; saving alone does not connect. PgUp/PgDn
changes sections, Tab moves through fields, Left/Right changes options,
Ctrl+U clears a field, and Esc closes the form.
The form shows an elapsed-time indicator while a test or connection attempt is
running, then shows validation errors or the test result in place. Saving closes
the form and confirms the profile name in the status line.

## Service name, SID and TNS

For ordinary Oracle services, use Hostname, Port and **Service Name**. If your
administrator gave you a SID instead, change Type to **SID**. For an Oracle Net
alias or a full connect descriptor, change Type to **TNS / Descriptor** and enter
it under Advanced. Set `TNS_ADMIN` to the directory containing `tnsnames.ora`
and `sqlnet.ora` before launch when those files are needed. In descriptor mode,
Oracle Net settings rather than the basic form fields control the address and
connection timeout. The per-call timeout still applies.

For TCPS, select **TCPS**, configure the Wallet directory and server-side
certificate support as instructed by your DBA. Server name matching is enabled.
TCPS/Wallet, external authentication, SID and administrative roles are
configurable but not covered by the standard Docker integration tests. Verify
them in a disposable environment before relying on them in DEV or QA.

## Credentials and environments

Create separate named profiles for DEV and QA. Never commit connection profiles,
Wallet files, database exports or real credentials. Profile TOML contains host,
user and connection metadata but not passwords, and may still be sensitive.
When Save password is on, the password is stored in Secret Service on Linux.
Headless SSH sessions often lack an unlocked keyring; leave Save password off
and enter the password at connection time.

Use least-privilege accounts. A test connection confirms authentication and a
simple query, **not** permission to inspect every schema or run every statement.
Object visibility is controlled by Oracle grants. Explorer's schema selection
affects inspection only; it does not run `ALTER SESSION SET CURRENT_SCHEMA`.

Autocommit is off, but Oracle DDL may commit implicitly. All open SQL files
share the same active connection and transaction. Commit or roll back explicitly
before changing environments. The UI prompts when it detects possible pending
work, but this is a conservative indicator, not a database transaction audit.

## Troubleshooting

- Missing Instant Client: install Basic or Basic Light for your architecture and
  set `LD_LIBRARY_PATH` to its directory before starting CauceDB.
- Failed connection: check VPN/routing, host, port, service/SID, credentials and
  Oracle listener status. `F5` shows the Oracle error in the UI.
- TNS alias not found: check `TNS_ADMIN` and the alias in `tnsnames.ora`.
- Wallet/TCPS failure: verify the Wallet location, file permissions, trust chain
  and server name with your DBA.
- Saved password unavailable: unlock/configure the Linux keyring or turn off
  Save password. CauceDB will not write a plaintext fallback.
- Query results appear incomplete: check the configured row and result-size
  limits in the connection form and README.

`F10` opens message history. Before sharing an error report, remove any host,
schema or query information your organization considers confidential.
