# ADR 0005: Host the Production Website on A2/cPanel

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Project maintainers

## Context

The discovery plan selected GitHub Pages before the owner established a
permanent production-hosting requirement. The owner has now selected the A2
Hosting cPanel account associated with `jamesjennison.net` and confirmed the
dedicated QuireForge subdomain.

The website remains static and version-controlled. GitHub continues to own
source, review, CI, issues, and application release artifacts, but it is no
longer the production website host.

## Decision

- Production URL: `https://quireforge.jamesjennison.net`.
- Production host: the owner's A2 Hosting cPanel account.
- Website source: `apps/website/` in `codeframe78/quireforge`.
- Generator: Astro static-site generation.
- Runtime: static files only; no persistent Node process, SSR, or database.
- Release binaries: GitHub Releases.
- GitHub Pages: disabled and not used as production or automatic fallback.
- Astro production base: `/` for the dedicated subdomain.

The exact document root and deployment mechanism remain pending the
authenticated capability audit. The preferred conditional mechanism is a
GitHub Actions build with protected-environment approval and SSH/`rsync`
delivery of a verified static artifact.

## Approval boundaries

This decision does not authorize:

- cPanel or SSH access;
- key or API-token creation;
- DNS, SSL, redirect, or document-root changes;
- staging or production deployment;
- GitHub environment, secret, or workflow-setting changes.

Each operation remains independently gated.

## Superseded decisions

This ADR supersedes the GitHub Pages URL/base entries in ADR 0003 and the active
deployment intent in `docs/GITHUB-PAGES.md`. Those records are retained as
historical discovery work. It does not alter the permanent product,
application, package, desktop-entry, repository, or XDG identities.

## Consequences

- Astro must be tested with `site` set to the production origin and a root base.
- Deployment credentials become supply-chain-sensitive and must live only in a
  protected GitHub environment or approved local credential facility.
- Source and `.git` remain outside public storage.
- Deployment must preserve a previous verified release and provide rollback.
- The main `www.jamesjennison.net` site and all mail-related DNS are out of
  deployment scope.
- Public DNS/TLS and authenticated hosting findings are maintained in
  `docs/A2-HOSTING-CAPABILITY-AUDIT.md`.
