# Compatibility

Status: desktop work through Milestone 15B is implemented and verified on the
discovery host. Milestone 15C's reviewed handoff and notification code is
implemented; its production Wayland launch and fixed-copy notification delivery
plus the complete XWayland handoff path are verified, while interactive Wayland
picker/attachment and true X11-login acceptance remain open.
Milestone 13 defines the Codex 0.145.0 integration contract and read-only
catalog; Milestones 14A–14C add fixed integration workflows, and Milestones
15A–15C add bounded local-file, conversation-image, and desktop-integration
contracts.

## Identity compatibility contract

The following values are reserved for implementation and must be validated in
the real toolchain rather than inferred from documentation alone:

| Surface                        | Target identity                            | Current validation state                                                                   |
| ------------------------------ | ------------------------------------------ | ------------------------------------------------------------------------------------------ |
| Product/window/desktop display | `QuireForge`                               | Verified in Tauri configuration and local Wayland launch                                   |
| Executable and Debian package  | `quireforge`                               | Unbundled executable verified; Debian package pending Milestone 20                         |
| Desktop entry filename         | `io.github.codeframe78.QuireForge.desktop` | Reverse-DNS contract retained; installed package output not yet validated                  |
| AppImage release basename      | `QuireForge`                               | Project release policy; verify the final workflow-renamed artifact                         |
| Application identifier         | `io.github.codeframe78.QuireForge`         | Verified as the running GTK/D-Bus application identity on Wayland                          |
| XDG directory leaf             | `quireforge`                               | Verified with isolated temporary XDG data; no personal persistent application data created |
| GitHub repository              | `codeframe78/quireforge`                   | Connected and renamed in place                                                             |
| Production website             | `https://quireforge.jamesjennison.net`     | Confirmed target; DNS/TLS present, site not deployed                                       |
| Website host                   | Cloudflare Pages                           | Public and owner-mediated account capabilities reviewed; project setup pending             |

Tauri, Cargo, React, TypeScript, Vite, and Astro configuration now consume the
applicable identity contracts. Package installation, desktop-entry output,
AppImage naming, and XDG persistence remain future validation obligations.

Validation sources:

- [Tauri configuration reference](https://v2.tauri.app/reference/config/#identifier):
  identifiers use reverse-domain notation and accept ASCII alphanumerics,
  hyphens, and periods; `mainBinaryName` independently controls the executable.
- [Freedesktop desktop-entry file naming](https://specifications.freedesktop.org/desktop-entry/latest/file-naming.html):
  application desktop filenames should use a reverse-DNS well-known name.
- [Freedesktop D-Bus activation](https://specifications.freedesktop.org/desktop-entry/latest/dbus.html):
  if enabled later, the D-Bus name must match the desktop filename without the
  `.desktop` suffix.
- [Debian Policy](https://www.debian.org/doc/debian-policy/ch-controlfields.html#s-f-source):
  package names use lowercase letters, digits, plus, minus, and period and begin
  with an alphanumeric character.
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/):
  configuration, data, cache, and state derive from their respective XDG base
  variables and documented home-relative defaults.
- [Astro configuration reference](https://docs.astro.build/en/reference/configuration-reference/#site):
  the production origin belongs in `site`; the dedicated subdomain uses the
  root base rather than a repository subpath.

The outer AppImage filename pattern
`QuireForge-{version}-{architecture}.AppImage` is a project release convention,
not an interface identity. The release workflow must rename and verify the
Tauri-produced artifact without changing its embedded executable or desktop
identity.

## Validated discovery environment

| Component                       | Observed value                         | Readiness                                  |
| ------------------------------- | -------------------------------------- | ------------------------------------------ |
| Operating system                | Ubuntu 26.04 LTS, x86_64               | Discovery only                             |
| Desktop session                 | GNOME on Wayland with XWayland display | Available                                  |
| Codex CLI                       | 0.145.0 standalone Linux build         | Current refresh; 0.144.6 fixtures retained |
| Codex authentication            | ChatGPT-managed                        | Available                                  |
| Active/default model            | GPT-5.6 Sol                            | Discovered from app-server                 |
| Node.js / npm                   | 22.22.1 / 9.2.0                        | Available                                  |
| Git / GitHub CLI                | 2.53.0 / 2.46.0                        | Available                                  |
| pnpm                            | 11.15.0                                | Available and pinned by the workspace      |
| Rust / Cargo                    | 1.97.1 / 1.97.1                        | Available; project minimum is Rust 1.88    |
| Tauri / CLI                     | Rust 2.11.5 / JavaScript 2.11.4        | Locked and locally built                   |
| WebKitGTK development package   | 2.52.3, API 4.1                        | Available and locally built against        |
| GTK / GLib development packages | GTK 3.24.52 / GLib 2.88.0              | Available and locally built against        |
| XDG desktop portal / GTK portal | Installed                              | Native picker feasible                     |

The portal executables live under the distribution's libexec directory rather
than the interactive shell `PATH`. The native dialog dependency and fixed
picker command compile into the verified release executable; interactive
portal selection remains a manual host check rather than an automated test.

The host is newer than the intended packaging baseline. Tauri recommends
building AppImages on the oldest supported compatible distribution to avoid
raising the minimum glibc requirement. Ubuntu 22.04 or Debian 12 are suitable
baseline examples in the [official AppImage guidance](https://v2.tauri.app/distribute/appimage/).

## Codex feature compatibility

| Capability                                 | CLI 0.145.0          | Intended application route                                                                                     | Classification                        |
| ------------------------------------------ | -------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| Detect version                             | Implemented          | Fixed `codex --version` probe                                                                                  | Stable official                       |
| List account-visible models/efforts        | Implemented          | Bounded `model/list` normalization                                                                             | Stable method on experimental server  |
| Apply model/effort to the next turn        | Yes                  | Revalidated `turn/start` model/effort overrides                                                                | Stable method on experimental server  |
| Agent-directed selector request lifecycle  | Contract validated   | `thread/start` dynamic-tool registration plus correlated `item/tool/call`; implementation remains Milestone 18 | Stable methods on experimental server |
| Start in an absolute local cwd             | Yes                  | `thread/start` / `turn/start`; CLI fallback                                                                    | Stable method + stable CLI            |
| Additional writable roots                  | Yes                  | sandbox `writableRoots`; CLI `--add-dir`                                                                       | Stable official                       |
| Stream turns, commands, plans, diffs       | Yes                  | app-server events                                                                                              | Stable methods on experimental server |
| Approve commands/file changes              | Yes                  | app-server server requests                                                                                     | Stable methods on experimental server |
| Resume/fork/archive/restore                | Yes                  | app-server; CLI fallback                                                                                       | Stable official                       |
| Search/list conversations                  | Partial              | stable thread title/cwd filters; experimental deeper paging                                                    | Mixed                                 |
| Codex-managed ChatGPT login                | Implemented          | Bounded app-server browser/device flow, cancel, logout, and normalized events                                  | Stable official                       |
| List skills by cwd                         | Yes                  | `skills/list`                                                                                                  | Stable method on experimental server  |
| Enable/disable skills                      | Implemented          | 14C native preview/confirmation, exact `skills/config/write`, and list postcondition                           | Stable method on experimental server  |
| List apps/connectors                       | Yes                  | `app/list`                                                                                                     | Stable method on experimental server  |
| Attach app to prompt                       | Implemented          | 14C native re-resolution and constructed documented `mention`/`app://` item                                    | Stable method on experimental server  |
| Attach local image to turn                 | Implemented          | 15B private PNG/JPEG staging and native-constructed `localImage` item                                           | Stable method on experimental server  |
| General connector authorization RPC        | Not established      | 14C confirmed official returned-URL handoff plus refreshed accessibility state                                 | Limited                               |
| MCP list/status/tools/auth                 | Yes                  | app-server + CLI                                                                                               | Stable official                       |
| MCP OAuth                                  | Implemented          | 14C native URL ownership and exact `mcpServer/oauthLogin/completed` correlation                                | Stable official                       |
| Plugin catalog via CLI JSON                | Implemented          | Bounded `plugin list --available --json` adapter                                                               | Supported CLI                         |
| Plugin install/remove via CLI JSON         | Implemented          | 14A fixed-command source review, one-use confirmation, and postcondition                                       | Supported CLI 0.145.x                 |
| Plugin app-server management               | Present              | Disabled in production                                                                                         | Under development                     |
| Marketplace add/list/upgrade/remove        | Implemented natively | Bounded list plus 14A confirmed fixed mutations; upgrade warns on mutable source                               | Supported CLI 0.145.x                 |
| Managed policy read                        | Yes                  | `configRequirements/read`                                                                                      | Stable method on experimental server  |
| Permission profile discovery               | Yes                  | `permissionProfile/list` with bounded summaries                                                                | Stable method on experimental server  |
| Client-owned dynamic tools                 | Contract validated   | `thread/start` registration and correlated `item/tool/call` server request                                     | Stable methods on experimental server |
| Integrated process API                     | Present              | Do not use as default terminal                                                                                 | Experimental/outside Codex sandbox    |
| Scheduled hosted tasks                     | Not established      | Defer                                                                                                          | Unsupported until discovered          |
| Repository status and staged/worktree diff | Git 2.53.0 available | Fixed native shell-free Git service over a revalidated attachment                                              | Implemented locally in Milestone 10A  |
| Stage, unstage, bounded revert, and commit | Git 2.53.0 available | Native-held preview/confirmation plans with fixed Git operations                                               | Implemented locally in Milestone 10B  |

Account-scoped inspection on 2026-07-19 returned a multi-page app directory
from both the default OpenAI catalog and ecosystem directory, with only a small
accessible subset. Catalog `isEnabled` and install-URL fields were much broader
than accessibility, so they are not proof of installation, authorization,
health, or eligibility. QuireForge must preserve that distinction and must not
publish the account-scoped rows or counts as a guaranteed catalog.

The same sanitized snapshot validated cwd-visible skill, configured
marketplace, available/installed plugin, and configured MCP collections. No
managed configuration requirements were returned. Counts, names, endpoints,
paths, and account metadata are intentionally omitted.

Milestone 13A did not repeat account-scoped catalog calls. Its 0.145.0 refresh
used local CLI help and generated schemas only, then validated deterministic
mock contracts. No personal connector, plugin, marketplace, skill, MCP,
configuration, permission-profile, or account record was captured.

Milestone 13B adds a version-gated 0.145.x runtime service. Connector, skill,
MCP, and effective-policy reads use reviewed app-server methods; plugins and
marketplaces use stable CLI JSON because the richer app-server plugin methods
remain under development. Every response is bounded and normalized into the
shared contract, and malformed or unavailable categories degrade independently.
Deterministic fixtures—not a personal account inventory—verified the complete
catalog path during implementation.

Milestone 14A enables only the reviewed plugin and marketplace mutation
capabilities. Preview and confirmation both depend on a fresh 0.145.x catalog,
effective policy, and ready normalized capability. Plugin sources must be a
reviewable local manifest, a credential-free HTTPS repository at a pinned
commit, or an exact-version package source. New repository marketplaces require
a pinned 40- or 64-hex reference. Configured marketplace upgrade remains
possible only for a safe repository source and is labeled mutable because the
next remote artifact is not present in list evidence. Every successful command
must return the exact closed JSON shape and satisfy a follow-up list
postcondition. Other CLI minors remain unavailable pending route review.
Built-in/default marketplace rows are read-only; removal is exposed only when
the fresh CLI record identifies an explicitly configured source.

Milestone 14C enables only the reviewed connector/MCP authorization and skill
configuration paths. Connector authorization uses the exact URL returned by
Codex because no general stable install RPC is established; completion requires
fresh accessible state. MCP OAuth uses the app-server login method and exact
completion-name correlation. Skill enable/disable uses only the native path
returned by `skills/list`, an expiring one-use confirmation, the exact effective
response, and a fresh list postcondition. Conversation mentions accept only
opaque normalized connector IDs and are converted natively to documented
`app://` mention paths after accessible/enabled/callable revalidation. Other
CLI minors and generic configuration or management paths remain unavailable.

Milestones 4–5 commit only the CLI 0.144.6 initialize, `model/list`, and stable
account-lifecycle generated schemas, their hashes, and sanitized deterministic
fixtures. The production adapter never treats fixture model names as account
availability. Live non-billable probes completed catalog normalization and a
non-mutating account read, then exited with no additional app-server process.
Login, browser authorization, logout, and turn execution were not invoked.

## Integration compatibility states

The application must use these states rather than a boolean:

- `compatible`: all known requirements are satisfied through supported
  interfaces.
- `incompatible`: a known requirement is not satisfied.
- `unknown`: metadata is insufficient.
- `authentication-required`: compatible but not connected.
- `runtime-missing`: a required executable/runtime is absent.
- `policy-blocked`: effective administrator/workspace requirements prohibit it.
- `unsupported-interface`: Codex exposes no supported management path.
- `disabled`: installed or accessible but locally disabled.
- `degraded`: installed but health checks or some components failed.

Compatibility evaluation includes Codex version and feature flags, Linux
support, binaries/runtimes, plan and workspace eligibility, administrator
approval, network/auth requirements, manifest validity, marketplace source
safety, and MCP transport support.

## Linux desktop compatibility policy

Initial target, subject to packaging validation:

- Primary development/QA: current Ubuntu LTS on GNOME Wayland and X11.
- Packaging baseline: the oldest Ubuntu release selected after WebKitGTK and
  Tauri validation, likely Ubuntu 22.04 or 24.04 rather than the discovery host.
- Initial architectures: x86_64; arm64 only after native package runners and
  desktop tests are available.
- Package formats: AppImage and Debian package in the release milestone.

No distribution is currently supported because the unbundled scaffold has been
verified on only one development host and no installation package exists.

## Native directory picker

Tauri 2's dialog plugin provides native directory selection. XDG desktop portal
and the GTK portal are installed on the discovery host. The application requests
a directory without using copy/import behavior and passes the result directly
to the Rust attachment service for validation; the frontend supplies no path.

Portal availability does not establish filesystem access. The service must
still check metadata, read/write expectations, mount state, Git state, and the
selected sandbox before saving an association or starting a task.

## Native file-preview compatibility

Milestone 15A reuses Tauri 2's native file dialog after a selected project is
available. The frontend passes only the app-owned project ID; the native path
is revalidated against the stored attachment and is never serialized. Readable
read-only attachments are eligible because preview does not mutate content.

The current renderer set is intentionally independent of desktop MIME handlers:
normalized UTF-8 text, bounded PNG/JPEG data, and metadata-only PDF recognition.
It does not depend on a PDF engine, image metadata helper, external command, or
editor association. APNG, unknown binary content, and files above the native
byte/dimension limits are refused; UTF-8 HTML/SVG source is shown only as inert
normalized text. The native picker compiles on the discovery Wayland host;
interactive Wayland and X11 behavior is recorded separately because a compile
or browser test is not display-session evidence.

## Conversation-image compatibility

The reviewed Codex CLI 0.145.0 `TurnStartParams` schema and current official
app-server documentation include `localImage` user inputs with native paths.
They do not establish a generic local-file input, so Milestone 15B supports only
bounded PNG/JPEG images and does not relabel PDF, text, or arbitrary files as
Codex attachments.

Native picker selections may come from outside an attached project only through
an explicit dialog choice; QuireForge copies validated bytes into private app
data and does not retain the source path. Browser drag/drop uses HTML `File`
bytes with Tauri default file-drop events disabled, so WebKitGTK does not become
a native path bridge. Source and staged paths remain native-only. Interactive
picker/drop behavior on both target Wayland and X11 sessions remains part of
the 15C manual compatibility gate.

## Desktop handoff and notification compatibility

Milestone 15C uses Tauri's official opener only after native code claims a
one-use preview action and revalidates the selected file. The external
destination class is the system default application; QuireForge does not
select, configure, or execute a custom editor. Notification delivery uses the
official Tauri notification plugin's Rust interface and the stable reverse-DNS
application identity. The webview retains no direct opener or notification
permission.

The discovery GNOME Wayland session exposes both `WAYLAND_DISPLAY=wayland-0`
and the shell-owned XWayland display `DISPLAY=:0`. `gnome-shell` 50.1 answers
the Freedesktop notification service. Results obtained with
`GDK_BACKEND=x11` on that display are labeled XWayland, not a separate GNOME
Xorg login. A true X11-session claim requires a real X11 login or equivalent
supported-session evidence.

The unbundled production artifact built through `pnpm desktop:build` launched
cleanly with `GDK_BACKEND=wayland`. Under `GDK_BACKEND=x11` on the host's
XWayland display, the same embedded artifact rendered the workspace and
completed attachment, native file selection, bounded README preview, explicit
default-application review, one-use opener call, and consumed-action UI state
against disposable QuireForge app data. The registered host viewer received
the revalidated file. A raw `cargo build --release` diagnostic artifact was not
accepted because it retained Tauri's development URL and therefore attempted
`127.0.0.1:1420`; rebuilding through the configured Tauri command corrected
that launch before evidence was recorded. No true X11 login was available.

The disabled-by-default `manual-notification-probe` Cargo feature supplied the
missing non-billable delivery check without creating a conversation or adding a
webview command. With the exact native flag and disposable app data, the
feature-enabled Tauri artifact sent the production completed-task title/body
through GNOME's Freedesktop notification service under `GDK_BACKEND=wayland`.
A filtered D-Bus capture contained only the `quireforge` application identity,
`Codex task completed`, and `Return to QuireForge to review the result.` The
configured normal build then replaced the probe artifact; a binary string check
confirmed that neither the probe flag nor its delivery log remained. This is
Wayland notification-service evidence, not a substitute for the still-open
interactive picker/attachment flow or a true X11 login. The discovery host has
no installed `/usr/share/xsessions` session descriptor, so a true X11 pass
requires a separately prepared supported host/session rather than relabeling
the available XWayland display.

## Website-host compatibility

The static Astro design is compatible with Cloudflare Pages static output,
preview deployments, custom domains, headers, and redirects. Account-level
inspection is complete; project-specific GitHub integration remains pending.
See the [Cloudflare audit](CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md).

GitHub Pages remains disabled and is not a production fallback.

## Known discovery limitations

- Only one local Linux environment and one Codex account were inspected.
- No real connector authorization or plugin installation was performed.
- No administrator-managed workspace was available for live policy tests.
- App-server protocol contracts can change with the installed Codex version.
- The plugin snapshot is account- and time-specific and cannot be treated as a
  public compatibility list.
- Public HTTP inspection is blocked by the host's current 403 behavior; no
  existing document-root or website-platform conclusion is possible.
- Unbundled Rust/Tauri release builds and the GTK/D-Bus runtime are verified on
  the discovery host; package installation and older-distribution compatibility
  remain unverified until the packaging milestone.
