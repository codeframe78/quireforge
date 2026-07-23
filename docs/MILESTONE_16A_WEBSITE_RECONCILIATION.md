# Milestone 16A: Private-Safe Website Reconciliation

- Status: Complete
- Date: 2026-07-22
- Production changes: None during 16A

## Outcome

The existing QuireForge Astro website and original visual identity were reused
for a public project presence while source, issue tracking, contribution
activity, and internal documentation remain private and unlinked.

The unimplemented Cloudflare Pages plan was superseded by a static Webuzo
origin. Cloudflare remains the authoritative DNS and proxied TLS/cache edge; it
does not host the website pages. Later, separately approved Milestones 16B–16D
staged and activated that architecture.

## Website changes

- Removed public source, issue, release, contribution, and repository links.
- Reframed detailed internal milestones as a high-level public roadmap.
- Retained the established product, documentation, security, support, and About
  routes.
- Preserved the original brand, layout, light/dark themes, reduced-motion
  behavior, and responsive design.
- Replaced the Cloudflare Pages `_headers` file with a domain-scoped Apache
  `.htaccess`.
- Added artifact checks for private-source markers, disallowed deployment
  files, canonical metadata, approved public hosts, and Apache directives.
- Kept the build fully static with no origin runtime, database, or background
  application process.

## Validation

- Frozen-lockfile installation and the complete repository validation passed.
- Astro, TypeScript, ESLint, Prettier, unit tests, production builds, Cargo,
  Clippy, and native Rust tests passed.
- The generated artifact passed route, link, host, canonical metadata, header,
  and private-content checks.
- Desktop and mobile Playwright and axe checks passed.
- Mobile and desktop Lighthouse audits reached 100 in Performance,
  Accessibility, Best Practices, and SEO.

The repository records acceptance outcomes only. Provider account identifiers,
server paths, certificate fingerprints, backup identifiers, deployment
manifests, and private diagnostics remain outside source control.

## Architecture and rollback

[ADR 0024](DECISIONS/0024-webuzo-static-website-hosting.md) is the authoritative
hosting decision. [Webuzo Deployment](WEBUZO-DEPLOYMENT.md) records the
repeatable, approval-gated deployment contract without embedding private server
state.

Source rollback is a Git revert of the website changes. Hosting rollback is a
separate, owner-approved operation covered by Milestones 16B–16D.
