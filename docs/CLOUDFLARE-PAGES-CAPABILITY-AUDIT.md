# Cloudflare Pages Capability Audit

- Audit date: 2026-07-19
- Production target: `https://quireforge.jamesjennison.net`
- Status: public documentation, DNS delegation, and owner-mediated read-only
  account audit complete; project creation remains approval-gated
- External changes made by Codex: none; the owner moved authoritative DNS to
  Cloudflare during the audit

## Decision summary

Cloudflare Pages is the selected production host for QuireForge's static Astro
website. The expected incremental hosting cost is zero when the site uses only
static assets. Static asset requests are documented as free and unlimited on
free and paid plans.

## Public DNS migration state

The `.net` registry delegates `jamesjennison.net` to the assigned Cloudflare
nameservers. Independent recursive resolvers showed mixed A2/Cloudflare
answers during normal delegation propagation. Both Cloudflare authoritative
servers agree on the new zone: apex, `www`, and mail-routing records exist,
but `quireforge.jamesjennison.net` has no A, AAAA, or CNAME record.

The QuireForge hostname will stop resolving as older A2 answers expire. Because
no production site exists, the correct remediation is not to restore the old
A2 origin automatically. Create and validate the Pages project and associate
the custom domain first; then add the exact Pages CNAME under a separate DNS
approval. Existing mail records must remain untouched.

## Account and zone findings

The owner supplied sanitized dashboard views; account and zone identifiers are
not retained. Findings are:

| Item | Status | Finding |
|---|---|---|
| Zone | Available | Active with Full DNS setup on Cloudflare Free |
| Registrar | Available | Remains external to Cloudflare |
| Workers & Pages | Available | No existing projects and no current usage |
| Pages project | Unavailable | Not created; no GitHub integration or deployment authorized |
| Universal SSL | Available | Managed wildcard primary and backup certificates are active |
| SSL mode | Available | Automatic mode currently selected `Full` for proxied origins |
| Always Use HTTPS | Available | Enabled zone-wide |
| TLS 1.3 | Available | Enabled |
| Opportunistic Encryption | Available | Enabled |
| Automatic HTTPS Rewrites | Available | Enabled |
| HSTS | Available but disabled | Keep disabled zone-wide during migration |
| Minimum TLS | Available | TLS 1.0 zone default; defer a zone-wide change pending main-site review |
| Certificate Transparency monitoring | Available but disabled | Optional future hardening |
| Two-factor authentication | Available | Enabled by the owner after the read-only audit; factor and recovery details are not retained |

The dashboard briefly exposed account and zone identifiers in an owner-supplied
screenshot. They are not authentication secrets, were not copied into project
files, and must not be reproduced in support material.

## Documented capability matrix

| Capability | Status | Finding |
|---|---|---|
| Static Astro output | Available | Git integration or direct upload can deploy prebuilt static assets |
| Custom subdomain on Cloudflare DNS | Available | Pages can activate the dedicated hostname in the authoritative Cloudflare zone |
| TLS | Available | Managed for an activated custom domain |
| Preview deployments | Available | Branch and same-repository pull-request previews receive immutable URLs |
| Rollback | Available | Earlier successful production deployments can be restored immediately |
| Custom headers | Available | Version-controlled `_headers`; 100-rule free-plan limit |
| Redirects | Available | Version-controlled `_redirects`; documented static/dynamic limits |
| Static requests/bandwidth | Available | Free and unlimited for requests that do not invoke Functions |
| Free builds | Available | 500 per month, one concurrent build, 20-minute timeout |
| Free file allowance | Available | 20,000 files, 25 MiB per individual asset |
| Custom domains | Available | Up to 100 per Pages project on Free |
| Functions | Not relevant | Dynamic runtime is deliberately excluded |
| Release binaries | Not relevant | AppImage/Debian artifacts remain on GitHub Releases |

## Remaining account-specific unknowns

- Global availability of the desired `quireforge.pages.dev` project name.
- GitHub integration installation and repository-selection state.
- Member roles, audit-log availability, and API-token inventory relevant to
  QuireForge.
- Whether Git integration or a GitHub Actions direct-upload workflow provides
  the narrower permission boundary for this account.

No credential should be pasted into chat. Account inspection requires a
separate approved method, preferably owner-provided sanitized screenshots or a
narrow read-only API token stored in an approved local credential facility.

## Security and migration constraints

- Keep two-factor authentication enabled before and after project creation,
  GitHub integration, token issuance, and QuireForge DNS changes.
- Add the Pages custom domain before creating its currently absent DNS record.
- Verify domain ownership and prevent dangling-CNAME takeover risk.
- Preserve the former A2 QuireForge origin value privately for rollback until
  Pages TLS and live behavior pass verification.
- Do not alter mail records or the main-site origin while completing Pages.
- Keep production and preview configuration separate; preview output is
  public unless protected and must remain `noindex`.
- Pin build dependencies and do not expose production credentials to pull
  requests from forks.

## Sources

- [Cloudflare Pages overview](https://developers.cloudflare.com/pages/)
- [Pages limits](https://developers.cloudflare.com/pages/platform/limits/)
- [Static and Functions pricing](https://developers.cloudflare.com/pages/functions/pricing/)
- [Custom domains](https://developers.cloudflare.com/pages/configuration/custom-domains/)
- [Preview deployments](https://developers.cloudflare.com/pages/configuration/preview-deployments/)
- [Rollbacks](https://developers.cloudflare.com/pages/configuration/rollbacks/)
- [Custom headers](https://developers.cloudflare.com/pages/configuration/headers/)
