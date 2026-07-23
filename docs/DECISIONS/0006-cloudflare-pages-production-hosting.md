# ADR 0006: Host the Production Website on Cloudflare Pages

- Status: Superseded by [ADR 0024](0024-webuzo-static-website-hosting.md)
- Date: 2026-07-19
- Decision owners: Project maintainers

## Context

QuireForge's website is a static Astro site. Cloudflare Pages provides native
Git-based builds, immutable preview deployments, production rollback, custom
headers and redirects, managed TLS, and global static delivery. The owner has
an existing Cloudflare account and selected Pages as the production host.

## Decision

- Production URL remains `https://quireforge.jamesjennison.net`.
- Cloudflare Pages is the production website host.
- Astro output remains entirely static; no Pages Functions are planned.
- GitHub remains authoritative for source, review, CI, issues, and release
  binaries.
- Cloudflare is authoritative DNS for `jamesjennison.net`; the owner completed
  that delegation separately after this decision was proposed.
- The QuireForge subdomain will use the supported Pages CNAME flow.
- GitHub Pages remains disabled and is not a fallback production host.

## Approval boundaries

This decision authorizes repository documentation only. It does not authorize
Cloudflare account access, project creation, GitHub app installation, DNS
changes, custom-domain activation, or deployment.

## Consequences

- Production Astro configuration uses the canonical origin and base `/`.
- Static requests are expected to remain within the Pages Free plan.
- Security headers and redirects are version controlled in the website source.
- Pull-request previews must not receive production secrets and must be marked
  `noindex`.
- Release packages remain on GitHub Releases, not Pages.
- DNS cutover requires a separately approved rollback plan and verification of
  TLS, canonical redirects, headers, assets, and the unaffected main site.

This ADR records the former plan. No Cloudflare Pages project or production
deployment was created. [ADR 0024](0024-webuzo-static-website-hosting.md) is the
authoritative production-hosting decision for QuireForge.
