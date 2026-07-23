# Website

Status: complete through Milestone 16D. The static Astro website is public
through a Webuzo-managed origin and Cloudflare DNS/proxy edge while source and
development activity remain private.

## Purpose

The QuireForge website explains the product direction, current availability,
compatibility boundaries, integrations, security and privacy principles,
public roadmap, releases, support state, and private-development boundary. It
clearly identifies QuireForge as an unofficial independent project and does not
imply OpenAI ownership or endorsement.

## Production contract

- URL: `https://quireforge.jamesjennison.net`.
- Origin host: a private Webuzo-managed Apache origin.
- Edge: Cloudflare authoritative DNS, proxy, cache, and TLS.
- Provider identifiers: resolved only during approved operations and never
  stored in the repository.
- Generator: Astro with TypeScript and static output.
- Production base: `/` on the dedicated subdomain.
- Runtime services and database: none.
- Source, issues, development activity, and internal documentation: private and
  absent from the public artifact.

The source generates Home, Features, Integrations, Downloads, Installation,
Documentation, Compatibility, Roadmap, Releases, Security, Development,
FAQ, Troubleshooting, About, and a custom 404. The production artifact is not
linked to the private source repository.

GitHub Pages remains disabled. The previously selected Cloudflare Pages project
was never created and is not a fallback host. Cloudflare remains authoritative
DNS and the public edge; Webuzo is authoritative for the application origin,
document root, certificate renewal, ownership, backup, and rollback.

## Public/private boundary

The public site may name QuireForge and James Jennison, explain the approved
product direction, state that no release exists, and present high-level planned
capabilities. It must not expose or link:

- the private source repository;
- issue, pull-request, release, or contribution systems;
- commit, branch, workflow, or detailed milestone activity;
- internal architecture and security documents;
- private paths, accounts, integrations, logs, credentials, or diagnostics.

The `/contributing/` route is retained to avoid a broken public URL, but it
honestly explains that public contribution intake is closed during private
development.

## Quality contract

- Reusable components and centralized design tokens.
- Light and dark themes with visible keyboard focus.
- Semantic HTML, screen-reader labels, and reduced-motion support.
- Responsive layouts and original QuireForge imagery.
- Canonical and social metadata, sitemap, robots, favicons, and custom 404.
- Minimal client-side JavaScript and no server-dependent controls.
- No fake testimonials, statistics, download counts, or compatibility claims.
- No repository-dependent browser request or privileged client token.

Targets are Lighthouse Performance 90+, Accessibility 95+, Best Practices 95+,
and SEO 95+. Any miss requires recorded evidence and remediation.

## Deployment artifact

The generated `apps/website/dist/` directory contains only the reviewed static
artifact. Its Apache `.htaccess`:

- disables indexes and content negotiation;
- assigns the custom 404;
- preserves Webuzo ACME validation paths;
- refuses noncanonical host aliases;
- enforces HTTPS for the exact approved hostname;
- supplies a strict static-site CSP and supporting security headers;
- uses immutable caching for hashed assets; and
- uses `no-transform` for HTML so Cloudflare does not inject scripts that would
  conflict with the reviewed CSP.

Live origin and edge HTTPS validation succeeded during Milestone 16C. The
artifact now sets domain-scoped HSTS with a one-year maximum age and
intentionally omits `includeSubDomains` and `preload`. No custom VirtualHost
configuration is used.

See [the Webuzo deployment plan](WEBUZO-DEPLOYMENT.md),
[ADR 0024](DECISIONS/0024-webuzo-static-website-hosting.md), and the
[Milestone 16C production report](MILESTONE_16C_PRODUCTION_ACTIVATION.md).
