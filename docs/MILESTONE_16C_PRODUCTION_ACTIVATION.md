# Milestone 16C: Production Activation

- Status: Complete
- Date: 2026-07-22
- Production URL: `https://quireforge.jamesjennison.net`
- Public source link: None

## Outcome

After separate owner approval, QuireForge became publicly available at its
canonical hostname. Webuzo hosts the static Apache origin. Cloudflare provides
authoritative DNS, proxying, public TLS, and caching; Cloudflare Pages does not
host the site.

Only the canonical QuireForge hostname was activated. No public source link,
secondary project hostname, analytics, application runtime, database, or
unrelated DNS change was introduced.

## Acceptance evidence

- Public DNS resolved through the approved proxied edge without exposing
  private origin details.
- Public edge and direct-origin TLS validation passed under Full (Strict).
- All public routes, redirects, the custom 404, canonical metadata, sitemap,
  robots file, security headers, and immutable asset caching passed.
- HSTS was enabled only after both edge and origin HTTPS passed; it is scoped to
  the QuireForge hostname and omits `includeSubDomains` and `preload`.
- Public HTML matched the reviewed origin output.
- Desktop and mobile Playwright checks passed every route without overflow or
  automatically detectable axe violations.
- Mobile and desktop Lighthouse audits reached 100 in Performance,
  Accessibility, Best Practices, and SEO.
- Pre-launch and post-launch recovery points and an application rollback
  rehearsal passed.

Provider record identifiers, origin addresses, server paths, account names,
certificate fingerprints, backup identifiers, and deployment manifests remain
outside source control.

## Rollback

Application rollback restores the retained prior artifact through the provider
and repeats the origin and public validation suite. If the origin must be
withdrawn, a separately approved DNS rollback targets only the canonical
QuireForge record after resolving it uniquely by hostname.

Milestone 16D subsequently enrolled the origin certificate in provider-managed
automatic renewal.
