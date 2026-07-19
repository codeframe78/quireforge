# ADR 0006: Host the Production Website on Cloudflare Pages

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Project maintainers

## Context

The A2/cPanel audit proved that the dedicated QuireForge subdomain can host a
static site, but deployment would require custom SSH synchronization, redirect
and header configuration, and project-owned rollback machinery. The audited
server exposes no shell Node toolchain and runs cPanel 110 on a CloudLinux 7
platform whose Extended Lifecycle Support ends January 1, 2027.

QuireForge's website is a static Astro site. Cloudflare Pages provides native
Git-based builds, immutable preview deployments, production rollback, custom
headers and redirects, managed TLS, and global static delivery. The owner has
an existing Cloudflare account and selected Pages as the replacement
production host.

## Decision

- Production URL remains `https://quireforge.jamesjennison.net`.
- Cloudflare Pages is the production website host.
- Astro output remains entirely static; no Pages Functions are planned.
- GitHub remains authoritative for source, review, CI, issues, and release
  binaries.
- Cloudflare is authoritative DNS for `jamesjennison.net`; the owner completed
  that delegation separately after this decision was proposed.
- A2 continues to host the main-site and mail origins unless a later decision
  changes those responsibilities.
- The QuireForge subdomain will use the supported Pages CNAME flow.
- GitHub Pages remains disabled and is not a fallback production host.

## Approval boundaries

This decision authorizes repository documentation only. It does not authorize
Cloudflare account access, project creation, GitHub app installation, DNS
changes, custom-domain activation, deployment, or deletion of the existing A2
document root or certificate.

## Consequences

- Production Astro configuration uses the canonical origin and base `/`.
- Static requests are expected to remain within the Pages Free plan.
- Security headers and redirects are version controlled in the website source.
- Pull-request previews must not receive production secrets and must be marked
  `noindex`.
- Release packages remain on GitHub Releases, not Pages.
- DNS cutover requires a separately approved rollback plan and verification of
  TLS, canonical redirects, headers, assets, and the unaffected main site.

## Superseded decisions

This ADR supersedes the active production-hosting decision in
[ADR 0005](0005-a2-production-hosting.md). The A2 audit remains a historical
record. Cloudflare is now authoritative DNS, while A2 remains the current
main-site and mail origin provider.
