# LinkQinShell Feature Roadmap

This roadmap tracks the functions described in `rust_slint_linkqinshell_dev_doc.md`.
Status values:

- `Implemented`: included in the current P0 code surface.
- `Planned`: intended for a near milestone, with architecture boundaries reserved.
- `Deferred`: intentionally outside P0, not forgotten.
- `Extension Pool`: requires user validation, technical spike, or service-side planning.

## P0 MVP

| Area | Feature | Status | Notes |
|---|---|---|---|
| Workspace | Rust + Slint multi-crate structure | Implemented | P0 crates created with long-term boundaries. |
| UI | Connection tree + tab workspace + SFTP/status panels | Implemented | Slint shell exists; live binding grows next. |
| Connection management | New/edit/delete/restore connection data model | Implemented | Soft-delete recovery window is modeled and tested. |
| Connection management | Search, favorite, recent, notes, tags, grouping | Planned | Fields and roadmap exist; UI list behavior follows storage integration. |
| Connection management | Drag sort, copy/cut/paste, import/export, batch move/edit | Deferred | Requires richer UI and repository operations. |
| SSH | Password authentication | Implemented | `ssh2` backend authenticates with decrypted password input. |
| SSH | Private key authentication with passphrase | Implemented | Path-based key auth is implemented; encrypted key content is modeled. |
| SSH | Host fingerprint first trust and changed-key blocking | Implemented | Host key model/status implemented; UI trust prompt wiring remains next. |
| SSH terminal | PTY shell session | Implemented | Backend opens PTY/shell; persistent streaming needs follow-up hardening. |
| SSH terminal | ANSI/VT parsing abstraction | Implemented | Basic ANSI stripping buffer exists; full terminal engine is P1. |
| SSH terminal | Input forwarding, clear, reconnect, retained output | Planned | State/interfaces exist; persistent channel lifecycle needs completion. |
| SFTP | Remote directory list | Implemented | `ssh2` SFTP list backend exists. |
| SFTP | Upload/download | Implemented | Basic file transfer backend exists. |
| SFTP | Rename, delete, mkdir | Implemented | Backend methods exist. |
| SFTP | Transfer conflict policy | Implemented | Overwrite/skip/rename policy is modeled and tested. |
| Security | Master password encryption | Implemented | Argon2 + AES-GCM payload format implemented and tested. |
| Security | Sensitive log redaction | Implemented | Redaction helper and tests cover common secret keys. |
| Storage | SQLite migrations | Implemented | P0 schema covers connections, groups, tags, host keys, tasks, settings, audit. |
| Storage | Connection repository | Implemented | Save/find/list/soft-delete/restore implemented. |
| Local data | Windows-first app data defaults | Implemented | `%LOCALAPPDATA%/LinkQinShell` via platform data dir. |
| Logging | `app.log` path and tracing setup | Planned | Path is defined; file appender wiring follows. |
| Packaging | Windows package | Deferred | Requires Rust toolchain and packaging setup. |

## P1 Near-Term Features

| Area | Feature | Status | Notes |
|---|---|---|---|
| Connection import | OpenSSH config import | Planned | Read-only import first: Host, HostName, User, Port, IdentityFile, ProxyJump. |
| Connection health | Online/offline/auth failure/timeout state | Planned | Uses existing session state model. |
| Tags | Tag filters and connection labels | Planned | Schema includes tags and links. |
| Quick connect | Temporary connection and save-after-success | Planned | UI callback reserved. |
| Terminal | Split panes | Deferred | Needs stable session/channel lifecycle first. |
| Terminal | Command input box, autocomplete, history, favorites | Planned | Core product differentiator after basic PTY stability. |
| Terminal | Copy modes, URL/path detection, right-click strategy | Deferred | Terminal profile work. |
| Terminal | Encoding selection and fallback fonts | Deferred | Required for Chinese/Powerline quality. |
| Terminal | Logs, search, export, retention | Planned | Buffer search exists; persistence follows audit/logging work. |
| SFTP | Drag upload/download | Planned | Requires OS drag/drop integration. |
| SFTP | Resume transfer | Planned | Large-file priority after basic transfer. |
| SFTP | File diff and external diff tool | Planned | Uses external tool detection. |
| SFTP | chmod/chown, hidden files, symlink handling | Planned | Backend operations after list/edit basics. |
| SFTP | Remote file search | Deferred | Needs cancellable async task center. |
| Operations | System information panel | Planned | Agentless command collection. |
| Operations | Service manager and remote log viewer | Planned | Requires dangerous-operation confirmations. |
| Monitoring | Threshold alerts and local notifications | Planned | Needs task/status center. |
| Monitoring | Batch health dashboard | Planned | Depends on connection health checks. |
| Security | Private key permission checks | Planned | Platform-specific filesystem behavior. |
| Security | Password reveal control and clipboard timeout | Planned | Needs master password unlock state in UI. |
| Security | Privacy mode | Planned | Disables histories/logs/recent writes. |
| Backup | Manual backup/restore wizard | Planned | Builds on SQLite backup policy. |
| UX | Task center for transfer/tunnel/batch/alerts | Planned | Schema has `task_runs`. |
| UX | Layout persistence and keyboard reachability | Planned | Slint app settings storage. |
| System integration | Tray, startup, notifications | Deferred | Windows first, cross-platform later. |
| Install | Portable Windows zip | Deferred | Requires packaging pipeline. |

## P2 Mid-Term Features

| Area | Feature | Status | Notes |
|---|---|---|---|
| Port forwarding | Local, remote, dynamic SOCKS5 | Deferred | Needs stable SSH channel multiplexing. |
| Proxy | HTTP, SOCKS5, ProxyJump, ProxyCommand, multi-hop | Planned | Data model has proxy enum; networking implementation deferred. |
| Batch operations | Batch connect, command, upload | Deferred | Requires task center and audit confirmations. |
| Remote desktop | RDP/VNC external client entry | Deferred | External detection and SSH tunnel linkage. |
| File transfer | SCP, ZModem, compression transfer | Deferred | Protocol-specific spikes needed. |
| Directory sync | One-way/two-way sync with preview | Deferred | Needs conflict preview and resumable transfers. |
| Monitoring | History curves and multi-location ping | Deferred | Requires storage retention strategy. |
| Docker | Container list/log/start/stop/exec/images/compose | Deferred | Agentless command implementation after SSH core. |
| Cron | Crontab/system timer management | Deferred | Dangerous edits require confirmation/audit. |
| Nginx/Apache | Site detection, syntax check, reload | Deferred | Targeted ops module. |
| Database entry | MySQL/PostgreSQL/Redis shortcuts and tunnels | Deferred | Not a full DB IDE in P0/P1. |
| Firewall | ufw/firewalld/iptables/nftables view/basic actions | Deferred | High-risk operations need audit. |
| Certificates | HTTPS certificate expiry checks | Deferred | Monitoring extension. |
| Sync | WebDAV/S3/Git/self-hosted encrypted sync | Deferred | Requires end-to-end encryption design. |
| Updates | Auto-update rollback and data migration | Deferred | Packaging pipeline dependency. |
| Compliance | Crash report switch, telemetry minimization, data export | Deferred | Product policy and UI needed. |

## P3 Extension Pool

| Area | Feature | Status | Notes |
|---|---|---|---|
| NAT traversal | Reverse tunnel, P2P, relay service | Extension Pool | Requires service-side architecture. |
| Acceleration | Dual-side TCP acceleration, node selection, compression | Extension Pool | Requires service-side nodes and protocol design. |
| Plugin platform | Plugin API, permissions, market, signing | Extension Pool | Sandbox and compatibility model required. |
| Team | Team spaces, shared connections, permissions | Extension Pool | Requires account/cloud backend. |
| Enterprise | Bastion mode, approvals, centralized audit | Extension Pool | Separate server product. |
| Licensing | Free/pro/enterprise, offline activation, device binding | Extension Pool | Must not block local core functions. |
| AI assistant | Command explanation, risk hints, log summaries | Extension Pool | Never auto-executes high-risk commands by default. |
| Cloud imports | Aliyun, Tencent Cloud, AWS, Azure, Google Cloud | Extension Pool | Provider credentials and least-privilege design needed. |
| External systems | DNS/CDN, object storage, Jira/Feishu/DingTalk/webhooks, CMDB | Extension Pool | Integration-specific scopes. |
| Advanced SSH | FIDO2/U2F, PKCS#11, certificates, Kerberos, X11, Mosh | Extension Pool | Protocol-specific compatibility spikes. |
| Legacy protocols | Telnet, Raw TCP, Serial enhanced tools | Extension Pool | Kept outside P0 to protect SSH/SFTP stability. |
| Kubernetes | Context/namespace/pod/log/exec/port-forward | Extension Pool | Start read-only if promoted. |
| Mobile/Web | Mobile status viewer, web management console | Extension Pool | Product-line expansion. |

## Promotion Rule

A deferred or extension feature enters active implementation only when it has:

1. A concrete user scenario.
2. Clear data model and state transitions.
3. Cross-platform feasibility.
4. Security, privacy, and audit boundaries.
5. Acceptance tests.
6. No dependency on unfinished service-side systems unless that system is the milestone.
