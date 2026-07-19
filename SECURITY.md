# Security Policy

QuireForge is pre-release software and is not ready to protect production or
sensitive workloads. No released version currently receives security updates.

| Version | Supported |
|---|---|
| Unreleased development work | Best effort only |
| Published releases | None |

## Reporting a vulnerability

Do not publish vulnerability details, credentials, private repository content,
connector data, or proof-of-concept exploits in a public issue.

GitHub private vulnerability reporting is not currently enabled for this
repository. Until the maintainers enable it, contact the repository owner
through the contact information on the
[codeframe78 GitHub profile](https://github.com/codeframe78) and request a
private reporting channel without including sensitive details in the initial
message. If private vulnerability reporting becomes available, use the
repository's **Security → Report a vulnerability** form instead.

Include, once a private channel is established:

- the affected revision or version;
- the operating system and installation method;
- a minimal reproduction and expected impact;
- whether credentials, local files, Codex sessions, Git repositories, or
  integration tools may be exposed; and
- suggested mitigations, if known.

Do not send real access tokens, private keys, OAuth codes, passwords, or
unredacted personal data. Revoke exposed credentials with their owner rather
than sharing them with QuireForge maintainers.

## Security boundaries

QuireForge must not own Codex login credentials or connector secrets. Codex,
the connector, or an operating-system credential facility remains authoritative
for those secrets. Application SQLite stores only QuireForge-owned metadata.

Reports are especially valuable for:

- directory identity confusion or access outside approved roots;
- command, shell, path, terminal-control, or deep-link injection;
- approval bypasses or misleading command presentation;
- unsafe Git/worktree cleanup or unintended file deletion;
- secret leakage through logs, fixtures, diagnostics, or support bundles;
- plugin, hook, marketplace, connector, or MCP supply-chain failures;
- unsafe Tauri capabilities, IPC commands, CSP, or update handling; and
- Cloudflare deployment or GitHub Actions trust-boundary weaknesses.

The maintainers will acknowledge a valid private report when practical, assess
impact, and coordinate remediation and disclosure. This project currently
offers no bug bounty or guaranteed response-time SLA.

See [the threat model](docs/THREAT-MODEL.md) for the current design analysis.
