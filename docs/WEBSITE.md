# Website

Status: architecture and hosting target established; Astro source and public
deployment do not exist yet.

## Purpose

The QuireForge website explains the product, installation, compatibility,
integrations, security/privacy, roadmap, releases, support, and contribution
process. It must clearly identify QuireForge as an unofficial community project
and must not imply OpenAI ownership or endorsement.

## Production contract

- URL: `https://quireforge.jamesjennison.net`.
- Host: the owner's A2 Hosting cPanel account.
- Source: `apps/website/` in `codeframe78/quireforge`.
- Generator: Astro with TypeScript and static output.
- Production base: `/` on the dedicated subdomain.
- Application downloads: GitHub Releases.
- Runtime services: none.

GitHub Pages remains disabled and is not a fallback production host. The exact
cPanel document root and deployment mechanism remain pending the separately
approved authenticated capability audit.

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

## Content and privacy boundary

Only reviewed, version-controlled public content enters the site build. Local
paths, SQLite data, Codex sessions, account/workspace details, installed
integrations, credentials, and unsanitized diagnostics are prohibited.

See [the A2 capability audit](A2-HOSTING-CAPABILITY-AUDIT.md),
[cPanel deployment architecture](CPANEL-DEPLOYMENT.md), and
[ADR 0005](DECISIONS/0005-a2-production-hosting.md).
