# Cloudflare Pages Deployment

Status: architecture selected; no project, integration, DNS change, or
deployment is authorized.

## Production contract

- URL: `https://quireforge.jamesjennison.net`.
- Host: Cloudflare Pages.
- Source: `apps/website/` in `codeframe78/quireforge`.
- Generator: Astro static output with production base `/`.
- Downloads: GitHub Releases.
- Runtime services and Pages Functions: none.

## Preferred workflow

The exact Git integration versus direct-upload choice follows the
account-specific audit. In either case:

1. Reconfirm owner-account two-factor authentication remains enabled.
2. Build and validate an exact reviewed commit.
3. Install pinned dependencies from the committed lockfile.
4. Run type, unit, accessibility, link, asset, 404, and secret checks.
5. Produce a deterministic artifact and checksum.
6. Deploy branches to previews; only the approved production branch may update
   the custom domain.
7. Apply reviewed `_headers` and `_redirects` files from source.
8. Verify the Pages deployment before changing Cloudflare DNS.
9. Create only the currently absent QuireForge subdomain record after separate
   approval.
10. Verify TLS, canonical host, headers, routes, assets, and the unaffected main
   site.
11. Retain both the prior Pages deployment and recoverable A2 configuration.

## Rollback

Before DNS cutover, rollback can restore the privately recorded prior A2 origin
in Cloudflare DNS. After a
successful cutover, ordinary application rollback selects the prior successful
Pages production deployment. DNS rollback remains documented until the A2
fallback is intentionally retired under separate approval.

## Credential boundary

Prefer the minimum-permission GitHub integration or Cloudflare token needed by
the approved workflow. Store credentials only in the provider integration or a
protected GitHub environment. Never place account IDs, tokens, or secrets in
Git, build artifacts, pull-request jobs, logs, or chat.

See [the capability audit](CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md) and
[ADR 0006](DECISIONS/0006-cloudflare-pages-production-hosting.md).
