# QuireForge

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/brand/quireforge-lockup-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/brand/quireforge-lockup.svg">
  <img alt="QuireForge — Build boldly. Work locally." src="assets/brand/quireforge-lockup.svg" width="620">
</picture>

> **Build boldly. Work locally.**

> [!IMPORTANT]
> This is an unofficial community project. It is not made, endorsed, supported,
> or distributed by OpenAI. QuireForge is an unofficial native Linux workspace
> for Codex.

QuireForge is an early-stage open-source project building a native graphical
Codex workspace for Linux. It works toward a direct, local-first project model:
user-selected directories remain in place and Codex operates against those
original directories through supported interfaces. The original Codex
discovery and QuireForge identity foundation are complete; production-hosting
feasibility is being reconciled for Cloudflare Pages. There is no application
package to install yet.

## Project status

- Supported distributions: none yet; Ubuntu support is being evaluated.
- Installation: not available during discovery.
- Website: planned at `https://quireforge.jamesjennison.net` on Cloudflare
  Pages; not deployed.
- Integration support: under validation against supported Codex interfaces.
- Completed milestone: Milestone 0 — existing-project, Codex, GitHub, A2, DNS,
  and Cloudflare Pages feasibility audit.
- Current milestone: Milestone 1 — residual rename/move/GitHub reconciliation;
  its core migration work is already complete locally.
- Upcoming milestone: Milestone 2 — brand and Cloudflare Pages website
  foundation.
- Known limitations: no desktop implementation, packages, releases, or public
  website exist yet.

## Discovery documents

- [Architecture](docs/ARCHITECTURE.md)
- [Codex integration findings](docs/CODEX-INTEGRATION.md)
- [Compatibility](docs/COMPATIBILITY.md)
- [Feature parity](docs/FEATURE-PARITY.md)
- [Threat model](docs/THREAT-MODEL.md)
- [A2 Hosting capability audit](docs/A2-HOSTING-CAPABILITY-AUDIT.md)
- [Cloudflare Pages capability audit](docs/CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md)
- [Cloudflare Pages deployment](docs/CLOUDFLARE-PAGES-DEPLOYMENT.md)
- [Superseded cPanel deployment architecture](docs/CPANEL-DEPLOYMENT.md)
- [Website architecture](docs/WEBSITE.md)
- [Superseded GitHub Pages plan](docs/GITHUB-PAGES.md)
- [Permanent identity decision](docs/DECISIONS/0003-permanent-quireforge-identity.md)
- [Brand sources and usage](assets/brand/README.md)
- [Roadmap](docs/ROADMAP.md)
- [Changelog](CHANGELOG.md)

The application will use only supported Codex and ChatGPT integration
mechanisms. It will not scrape ChatGPT, copy browser tokens, reverse engineer a
proprietary desktop application, or present account-specific integrations as a
guaranteed public catalog.

## Permanent identity

The repository is `codeframe78/quireforge`. Future application and
packaging work must use `quireforge` for the executable and Debian package,
`QuireForge` for the desktop display name and AppImage basename, and
`io.github.codeframe78.QuireForge` as the application identifier. Its syntax is
validated for Tauri and freedesktop application identity; functional bundle,
GTK, and packaging wiring remains an implementation-milestone test obligation.
The canonical desktop entry is
`io.github.codeframe78.QuireForge.desktop`; its `Exec` target remains
`quireforge`.

The production website target is
`https://quireforge.jamesjennison.net`, hosted as a static Astro site on
Cloudflare Pages. Cloudflare is authoritative DNS; A2 retains the main-site and
mail origins unless separately changed. GitHub remains the source, CI, issue,
and release host. GitHub Pages is disabled and is not the production host.

Application-owned files will use the XDG locations `~/.config/quireforge`,
`~/.local/share/quireforge`, `~/.cache/quireforge`, and, where needed,
`~/.local/state/quireforge`. Codex-owned authentication, configuration, and
session storage are outside this identity migration.
