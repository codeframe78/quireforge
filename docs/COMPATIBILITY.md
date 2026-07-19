# Compatibility

Status: Milestone 0 discovery; implementation has not started.

## Identity compatibility contract

The following values are reserved for implementation and must be validated in
the real toolchain rather than inferred from documentation alone:

| Surface | Target identity | Current validation state |
|---|---|---|
| Product/window/desktop display | `QuireForge` | Valid Tauri product/display name |
| Executable and Debian package | `quireforge` | Valid Linux binary and Debian package form |
| Desktop entry filename | `io.github.codeframe78.QuireForge.desktop` | Valid and reverse-DNS aligned; replaces the initial short-name proposal |
| AppImage release basename | `QuireForge` | Project release policy; verify the final workflow-renamed artifact |
| Application identifier | `io.github.codeframe78.QuireForge` | Valid Tauri and freedesktop identifier form; functional wiring pending |
| XDG directory leaf | `quireforge` | Valid; honor XDG environment overrides |
| GitHub repository | `codeframe78/quireforge` | Connected and renamed in place |
| Production website | `https://quireforge.jamesjennison.net` | Confirmed target; DNS/TLS present, site not deployed |
| Website host | Cloudflare Pages | Public and owner-mediated account capabilities reviewed; project setup pending |

No Tauri, Cargo, JavaScript, Astro, package, or desktop-entry configuration
exists yet, so this table is a future implementation contract rather than a
claim that those artifacts have already been migrated.

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

| Component | Observed value | Readiness |
|---|---|---|
| Operating system | Ubuntu 26.04 LTS, x86_64 | Discovery only |
| Desktop session | GNOME on Wayland with XWayland display | Available |
| Codex CLI | 0.144.6 standalone Linux build | Available/current at inspection |
| Codex authentication | ChatGPT-managed | Available |
| Active/default model | GPT-5.6 Sol | Discovered from app-server |
| Node.js / npm | 22.22.1 / 9.2.0 | Available |
| Git / GitHub CLI | 2.53.0 / 2.46.0 | Available |
| Rust / Cargo | Not installed | Blocking desktop builds |
| pnpm | Not installed | Tooling decision pending |
| Tauri WebKitGTK development package | Not installed | Blocking desktop builds |
| Tauri Linux development packages | Partially installed | Blocking desktop builds |
| XDG desktop portal / GTK portal | Installed | Native picker feasible |

The portal executables live under the distribution's libexec directory rather
than the interactive shell `PATH`. Runtime packages are installed, but the
development headers required to compile Tauri are not.

The host is newer than the intended packaging baseline. Tauri recommends
building AppImages on the oldest supported compatible distribution to avoid
raising the minimum glibc requirement. Ubuntu 22.04 or Debian 12 are suitable
baseline examples in the [official AppImage guidance](https://v2.tauri.app/distribute/appimage/).

## Codex feature compatibility

| Capability | CLI 0.144.6 | Intended application route | Classification |
|---|---|---|---|
| Detect version and features | Yes | CLI + app-server | Stable official |
| List account-visible models/efforts | Yes | `model/list` | Stable method on experimental server |
| Start in an absolute local cwd | Yes | `thread/start` / `turn/start`; CLI fallback | Stable method + stable CLI |
| Additional writable roots | Yes | sandbox `writableRoots`; CLI `--add-dir` | Stable official |
| Stream turns, commands, plans, diffs | Yes | app-server events | Stable methods on experimental server |
| Approve commands/file changes | Yes | app-server server requests | Stable methods on experimental server |
| Resume/fork/archive/restore | Yes | app-server; CLI fallback | Stable official |
| Search/list conversations | Partial | stable thread title/cwd filters; experimental deeper paging | Mixed |
| Codex-managed ChatGPT login | Yes | app-server browser/device flow; CLI fallback | Stable official |
| List skills by cwd | Yes | `skills/list` | Stable method on experimental server |
| Enable/disable skills | Yes | `skills/config/write` | Stable method on experimental server |
| List apps/connectors | Yes | `app/list` | Stable method on experimental server |
| Attach app to prompt | Yes | documented `mention` item | Stable method on experimental server |
| General connector authorization RPC | Not established | Official returned URL/browser handoff | Limited |
| MCP list/status/tools/auth | Yes | app-server + CLI | Stable official |
| MCP OAuth | Yes | app-server/CLI official flow | Stable official |
| Plugin catalog via CLI JSON | Yes | CLI adapter | Supported CLI |
| Plugin install/remove via CLI JSON | Yes | CLI adapter with confirmation | Supported CLI |
| Plugin app-server management | Present | Disabled in production | Under development |
| Marketplace add/list/upgrade/remove | Yes | CLI adapter initially | Supported CLI |
| Managed policy read | Yes | `configRequirements/read` | Stable method on experimental server |
| Integrated process API | Present | Do not use as default terminal | Experimental/outside Codex sandbox |
| Scheduled hosted tasks | Not established | Defer | Unsupported until discovered |

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

Initial target, subject to Milestone 3 and packaging validation:

- Primary development/QA: current Ubuntu LTS on GNOME Wayland and X11.
- Packaging baseline: the oldest Ubuntu release selected after WebKitGTK and
  Tauri validation, likely Ubuntu 22.04 or 24.04 rather than the discovery host.
- Initial architectures: x86_64; arm64 only after native package runners and
  desktop tests are available.
- Package formats: AppImage and Debian package in the release milestone.

No distribution is currently supported because no application build exists.

## Native directory picker

Tauri 2's dialog plugin supports native directory selection. XDG desktop portal
and the GTK portal are installed on the discovery host. The application will
request a directory path without using copy/import behavior and pass the result
to the Rust attachment service for validation.

Portal availability does not establish filesystem access. The service must
still check metadata, read/write expectations, mount state, Git state, and the
selected sandbox before saving an association or starting a task.

## Website-host compatibility

The static Astro design is compatible with Cloudflare Pages static output,
preview deployments, custom domains, headers, and redirects. Account-level
inspection is complete; project-specific GitHub integration remains pending. The earlier
A2/cPanel design was compatible with ordinary cPanel static hosting. Its
authenticated audit confirmed an isolated document root, TLS 1.2/1.3,
LiteSpeed, SSH/`rsync`, Git, AutoSSL, ModSecurity, and account resource limits;
the empty endpoint currently returns the expected no-index 403. Atomic release
switching and staging were not tested before that design was superseded. See the
[Cloudflare audit](CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md) and historical
[A2 audit](A2-HOSTING-CAPABILITY-AUDIT.md).

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
- Rust/Tauri builds cannot run until the missing toolchain and system packages
  are installed in a later approved milestone.
