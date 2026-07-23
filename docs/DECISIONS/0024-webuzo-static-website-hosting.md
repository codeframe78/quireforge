# ADR 0024: Host the Static Website on Webuzo

- Status: Accepted; public-source boundary amended on 2026-07-23 by
  [ADR 0027](0027-public-source-and-runner-boundary.md)
- Date: 2026-07-22
- Decision owners: Project owner and maintainers
- Supersedes: [ADR 0006](0006-cloudflare-pages-production-hosting.md)

## Context

QuireForge already has a validated static Astro website and a canonical public
hostname. The owner selected the existing Webuzo production server as the
hosting authority for project websites while retaining Cloudflare as the
authoritative DNS, TLS edge, and proxy.

The former Cloudflare Pages project was never created. Read-only discovery
verified a compatible Webuzo-managed Apache origin and recoverable hosting
workflow. Provider account names, server paths, certificate details, DNS record
identifiers, and backup identifiers are deliberately excluded from this
decision.

## Decision

- Production remains `https://quireforge.jamesjennison.net`.
- Webuzo is authoritative for the subdomain, document root, Apache
  configuration, origin SSL, ownership, backup, and rollback.
- Cloudflare remains authoritative DNS and provides the proxied public TLS and
  cache edge using Full (Strict) origin validation.
- Astro continues to produce a static artifact with no origin runtime,
  database, reverse proxy, scheduled application task, or open application
  port.
- Source, build dependencies, Git metadata, credentials, and internal
  documentation remain outside the public document root.
- A reviewed artifact is built outside public storage and promoted only to the
  exact document root Webuzo reports for the approved hostname.
- Apache behavior is limited to the artifact's domain-scoped `.htaccess`; no
  generated VirtualHost or global web-server configuration is hand-edited.
- The public source repository is not synchronized into the website document
  root. The static site may link to it, but website deployment remains an
  independent owner-hosted operation.
- GitHub Pages and Cloudflare Pages remain disabled and are not fallbacks.

## Approval boundaries

This decision authorizes source and documentation changes only. It does not
authorize Webuzo domain or document-root creation, certificate issuance or
replacement, DNS changes, Cloudflare proxy changes, staging, production
deployment, GitHub push, or release publication. Each remains a separate owner
approval gate. Milestones 16B–16D later recorded those hosting approvals and
acceptance outcomes without broadening this decision.

## Consequences

- Cloudflare Pages preview and rollback features are replaced with controlled
  artifact manifests, origin-only staging validation, Restic snapshots, and a
  retained previous document root.
- The production artifact carries Apache-compatible headers rather than a
  Cloudflare Pages `_headers` file.
- HSTS is added only after the origin and proxied hostname are verified.
- Webuzo-created dormant aliases receive no DNS records and the artifact
  refuses to serve duplicate content for noncanonical host headers.
- Deployment does not require a GitHub app, GitHub secret, Cloudflare Pages
  token, persistent Node process, or service restart.
