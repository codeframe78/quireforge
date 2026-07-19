# Website

Status: Milestone 2 static Astro foundation implemented and verified locally;
Cloudflare project creation and public deployment remain approval-gated.

## Purpose

The QuireForge website explains the product, installation, compatibility,
integrations, security/privacy, roadmap, releases, support, and contribution
process. It must clearly identify QuireForge as an unofficial community project
and must not imply OpenAI ownership or endorsement.

## Production contract

- URL: `https://quireforge.jamesjennison.net`.
- Host: Cloudflare Pages.
- Source: `apps/website/` in `codeframe78/quireforge`.
- Generator: Astro with TypeScript and static output.
- Production base: `/` on the dedicated subdomain.
- Application downloads: GitHub Releases.
- Runtime services: none.

The source currently generates Home, Features, Integrations, Downloads,
Installation, Documentation, Compatibility, Roadmap, Releases, Security,
Contributing, FAQ, Troubleshooting, About, and a custom 404. The production
artifact is not yet public.

GitHub Pages remains disabled and is not a fallback production host. Cloudflare
is authoritative DNS; A2 keeps the main-site and mail origins unless separately
changed. The currently absent QuireForge record will use Cloudflare's supported
Pages CNAME flow after separately approved project setup and validation.

## Information architecture

The production site includes:

1. Home
2. Features
3. Integrations
4. Downloads
5. Installation
6. Documentation
7. Compatibility
8. Roadmap
9. Releases/changelog
10. Security and privacy
11. Contributing
12. FAQ and troubleshooting
13. About
14. GitHub repository link

Integration pages explain categories, supported-interface boundaries,
authorization, scope, permissions, and supply-chain risk. They never expose the
owner's locally installed integrations or present an account snapshot as a
guaranteed catalog.

## Quality contract

- Reusable components and centralized design tokens.
- Light and dark themes with visible keyboard focus.
- Semantic HTML, screen-reader labels, and reduced-motion support.
- Responsive layouts and images.
- Original QuireForge assets only; no OpenAI or ChatGPT visual assets.
- Canonical/social metadata, sitemap, robots, favicon, and custom 404.
- Minimal client-side JavaScript and no server-dependent controls.
- No fake testimonials, statistics, download counts, or compatibility claims.

Targets are Lighthouse Performance 90+, Accessibility 95+, Best Practices 95+,
and SEO 95+. Any miss requires recorded evidence and remediation.

## Implemented foundation

- Astro static output with strict TypeScript and a committed pnpm lockfile.
- Central content and identity data, reusable layout/header/footer/hero/status
  components, and layered design tokens.
- System-aware light and dark themes with a persistent user toggle.
- Responsive navigation, visible focus, skip navigation, reduced-motion
  handling, semantic landmarks, and an axe-tested baseline.
- Canonical and social metadata, sitemap, robots policy, manifest, favicon,
  responsive brand treatment, and a root custom 404.
- A version-controlled Cloudflare `_headers` policy with no HSTS until the live
  hostname and redirects are verified.
- Deterministic unit, route, link, asset, content, responsive, theme, and
  accessibility checks.

No analytics, telemetry, external font request, server runtime, Pages Function,
or Cloudflare credential is present. Downloads and installation pages state
that no package or release exists.

Local commands and prerequisites are in [Building](BUILDING.md); validation is
in [Testing](TESTING.md).

## Content and privacy boundary

Only reviewed, version-controlled public content enters the site build. Local
paths, SQLite data, Codex sessions, account/workspace details, installed
integrations, credentials, and unsanitized diagnostics are prohibited.

See [the Cloudflare Pages capability audit](CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md),
[deployment architecture](CLOUDFLARE-PAGES-DEPLOYMENT.md), and
[ADR 0006](DECISIONS/0006-cloudflare-pages-production-hosting.md). The
[A2 audit](A2-HOSTING-CAPABILITY-AUDIT.md) is retained as migration evidence.
