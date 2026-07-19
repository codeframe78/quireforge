# Cloudflare Pages Deployment

Status: architecture and local static artifact established; no project,
integration, custom-domain change, DNS change, or deployment is authorized.

## Production contract

- URL: `https://quireforge.jamesjennison.net`.
- Host: Cloudflare Pages.
- Source: `apps/website/` in `codeframe78/quireforge`.
- Generator: Astro static output with production base `/`.
- Downloads: GitHub Releases.
- Runtime services and Pages Functions: none.

## Expected build settings

- Repository: `codeframe78/quireforge`
- Production branch: `main`
- Root directory: `apps/website`
- Build command: `pnpm build`
- Output directory: `dist`
- Node.js: `22.22.1` (or a later compatible Node 22 release)
- pnpm: `11.15.0`, selected from the committed package-manager contract

These settings are documentation, not a created Cloudflare project. Confirm
them again against the current Pages UI immediately before an approved setup.

## Preferred workflow

The exact Git integration versus direct-upload choice follows the
account-specific audit. In either case:

1. Reconfirm owner-account two-factor authentication remains enabled.
2. Build and validate an exact reviewed commit.
3. Install pinned dependencies from the committed lockfile.
4. Run type, unit, accessibility, link, asset, 404, and secret checks.
5. Produce and inspect the static artifact; record a checksum for an approved
   deployment candidate.
6. Deploy branches to previews; only the approved production branch may update
   the custom domain.
7. Apply reviewed `_headers` and `_redirects` files from source.
8. Verify the Pages deployment before changing Cloudflare DNS.
9. Create only the currently absent QuireForge subdomain record after separate
   approval.
10. Verify TLS, canonical host, headers, routes, assets, and the unaffected main
   site.
11. Retain the previous successful Pages deployment for rollback. A2 is not a
    QuireForge deployment target.

## Rollback

Before the first deployment, rollback means making no DNS change and leaving
the currently absent QuireForge hostname untouched. After a successful cutover,
ordinary application rollback selects the prior successful Pages production
deployment. If the first cutover fails, remove only the QuireForge custom-domain
mapping or record under explicit approval; do not alter the main site or mail.

## Credential boundary

Prefer the minimum-permission GitHub integration or Cloudflare token needed by
the approved workflow. Store credentials only in the provider integration or a
protected GitHub environment. Never place account IDs, tokens, or secrets in
Git, build artifacts, pull-request jobs, logs, or chat.

See [the capability audit](CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md) and
[ADR 0006](DECISIONS/0006-cloudflare-pages-production-hosting.md).
