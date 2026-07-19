# Feature Parity Matrix

The objective is comparable workflow coverage, not pixel-level imitation or
access to proprietary implementation details. Windows/ChatGPT behavior below is
based only on public documentation and observable supported concepts.

QuireForge is an unofficial client. Product identity does not alter official
Codex commands, protocol types, integration IDs, or compatibility boundaries.

| Product capability | Public desktop behavior | Supported official interface | Local project responsibility | Milestone/status |
|---|---|---|---|---|
| Projects linked to local folders | Local project gives chats one or more folders | Absolute `cwd`, `--cd`, sandbox roots | Project DB, native picker, identity checks | M6 / feasible |
| Original directory used in place | Project folder is working directory | App-server/CLI cwd | Never copy/import; relink and detach UX | M6 / feasible |
| Multiple folder roots | Public project docs say one or more folders | Writable roots / `--add-dir` | Persistent association model and approvals | M6 / schema-first |
| Model and reasoning picker | Desktop exposes available choices | `model/list` | Discovery normalized in M4; controls remain M7 | M4 adapter complete / M7 UI planned |
| Streaming conversation | Rich item and event stream | App-server v2 | Base event contract in M4; turn normalization/rendering remains M7 | M4 boundary complete / M7 planned |
| Stop task | Interactive interruption | `turn/interrupt` | Task controls and recovery | M7 / feasible |
| Resume/fork/archive/restore | Documented conversation organization | Thread RPCs and CLI commands | Project association and UX | M8 / feasible |
| Search conversations | Search past chats/projects | Stable title/cwd filters; deeper paging mixed | Local metadata index without copying authoritative content | M8 / partial |
| Command/file approvals | Desktop asks before protected actions | Approval server requests | Safe, scoped approval UI | M9 / feasible |
| Plans, commands, and diffs | Desktop activity and diff views | Normalized app-server events | Presentation and review state | M7/M10 / feasible |
| Git stage/revert/commit/push/PR | Public desktop docs describe these controls | Git CLI is authoritative | Explicit Git service and confirmations | M10 / local |
| Worktrees | Public desktop supports managed/permanent worktrees | Git CLI; Codex cwd | Safe lifecycle and cleanup | M11 / local |
| Concurrent tasks | Multiple chats/tasks | Multiple threads/processes | Scheduler/status dashboard | M11 / feasible |
| Integrated terminal | Public desktop terminal/actions | No need to use Codex process API | Dedicated PTY service | M12 / local |
| Apps/connectors directory | Desktop exposes account-eligible apps | `app/list` | Normalized catalog, filters, browser handoff | M13/M14 / feasible |
| App prompt attachment | Mention an installed app | `mention` with `app://` path | Composer integration | M14 / feasible |
| Connector authorization | Official service/workspace authorization | Returned install URL; no general stable install RPC established | Browser handoff and status refresh | M14 / limited |
| Plugins directory | Desktop reads marketplaces | CLI JSON; app-server RPCs under development | CLI fallback and honest capability labels | M13/M14 / feasible with fallback |
| Plugin install/remove/update | Desktop plugin lifecycle | CLI add/remove; marketplace upgrade | Preview, progress, confirmation, health | M14 / feasible with gaps |
| Plugin enable/disable | Desktop stores plugin state in config | Effective config; exact stable mutation path requires contract validation | Scoped settings UI | M14 / requires validation |
| Skills | Built-in/local/plugin skills | `skills/list` and config write | Scope/provenance UI | M13/M14 / feasible |
| MCP servers and OAuth | Desktop/CLI MCP support | App-server and CLI MCP interfaces | Health/auth UI without secret ownership | M13/M14 / feasible |
| Managed restrictions | Enterprise requirements | `configRequirements/read` | Policy-blocked states | M13/M18 / feasible |
| Local environment actions | Desktop `.codex` setup/actions | Project config is readable | Safe action UI and terminal execution | Later / partly local |
| File previews/notifications/editor | Desktop integration | OS/Tauri APIs | Native Linux implementation | M15 / local |
| Scheduled tasks | Desktop may expose eligible scheduling | No validated interface in M0 | Do not fabricate; local scheduling separately labeled | M17 / deferred |
| Browser-token reuse/private catalog | Not a supported project goal | None | Prohibited | Unsupported |
| Exact Windows application reproduction | Proprietary implementation | None | Prohibited | Unsupported |

## Classification rules

Every release-facing feature must carry one of these labels:

1. Stable official interface.
2. Experimental official interface.
3. Implemented locally by this project.
4. Deferred because no supported interface exists.

Mixed features must expose the weakest relevant dependency. For example, a
locally polished plugin page backed by an under-development RPC remains
experimental unless it uses a stable CLI fallback.
