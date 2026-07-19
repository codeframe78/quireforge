# A2 Hosting Capability Audit

- Audit date: 2026-07-19
- Production target: `https://quireforge.jamesjennison.net`
- Status: public, local, and authenticated read-only inspection complete
- Hosting changes made: a dedicated audit public key was authorized; no site,
  DNS, SSL, repository, deployment, or content setting changed
- Production disposition: superseded by Cloudflare Pages; A2 remains the
  main-site/mail origin and a recoverable prior QuireForge origin

## Scope and redaction

This report evaluates whether the owner's A2 Hosting cPanel account can safely
host QuireForge's statically generated Astro website. Evidence comes from the
local QuireForge checkout, read-only GitHub metadata, public DNS and HTTP/TLS,
authoritative documentation, and an approval-gated read-only SSH/cPanel audit.

Raw IP addresses, mail-routing values, account usernames, home paths, account
IDs, credentials, host-key material, and unrelated hosting content are
intentionally omitted. A dedicated passphrase-protected local audit key was
authorized in cPanel and strict host checking pinned the repeatedly observed
Ed25519 server key using trust on first use. Independent confirmation of that
server fingerprint by A2 remains desirable. No API token, DNS mutation,
upload, content edit, package installation, or deployment occurred.

Status labels mean:

- **Available**: directly observed for this account or public endpoint.
- **Unavailable**: directly tested and absent or rejected.
- **Provider-disabled**: an authenticated account view establishes that the
  provider disabled the feature.
- **Requires support**: provider action is required.
- **Unknown**: not safely established yet.
- **Not relevant**: unnecessary for the static QuireForge site.

## Executive summary

The dedicated subdomain is already isolated from the main site with its own
document root outside `public_html`, a distinct Let's Encrypt AutoSSL
certificate, and ModSecurity enabled. Its root contains only provider-managed
ACME/CGI directories and no index file, which explains the current
`403 Forbidden`; the main document root is similarly unpopulated except for a
small `robots.txt`. LiteSpeed serves HTTP/2 over TLS 1.2/1.3.

The account is an A2 `Drive 2020` shared-hosting plan on cPanel 110 and an
EL7-derived CloudLinux kernel. SSH, Git Version Control, `rsync`, SFTP/SCP,
archives, AutoSSL, backups, cron, logs, ModSecurity, API tokens, and 2FA are
available. Node/npm/Corepack/pnpm are not exposed in the shell and persistent
Passenger applications are provider-disabled. cPanel 110/CloudLinux 7 is in
Extended Lifecycle Support only through January 1, 2027, making provider
migration timing a material production risk.

The audit established that GitHub Actions plus SSH/`rsync` was viable but
required custom release and rollback machinery. The owner subsequently selected
Cloudflare Pages as the production host in ADR 0006. The audit key is not a
deployment key, and no A2 deployment is authorized.

## Preserved project and GitHub baseline

| Item | Finding |
|---|---|
| Local checkout | Intact at `/mnt/faststorage/quireforge` |
| Audit branch | `docs/a2-capability-audit`, based on `04c36f7` |
| Working tree at audit start | Clean |
| Git integrity | `git fsck --full --no-dangling` passed |
| Local branches | `main`, discovery, rename, and audit branches preserved |
| Tags / submodules / linked worktrees | No tags or submodules; one intact working tree |
| GitHub repository | Public `codeframe78/quireforge` |
| Default branch | `main` |
| Published branches | `main` and the earlier discovery branch only |
| Issues / pull requests / releases | None open/published |
| Actions workflows / environments | None configured |
| Actions secrets / variables | None configured |
| Deploy keys / webhooks | None configured |
| Default workflow token | Read-only; cannot approve pull-request reviews |
| Secret scanning / push protection | Enabled |
| Dependabot security updates | Disabled; no dependency manifest/config exists yet |
| Vulnerability alerts | Disabled or unavailable to the current authenticated view |
| Main-branch protection | Not configured |
| GitHub Apps / linked Packages | Not enumerable with the current user token; no repository integration is assumed |
| GitHub Pages | Disabled |
| QuireForge executable/process | No installed executable or running application process detected |
| QuireForge/old-name XDG data | No configuration, data, cache, state, or desktop-entry artifacts detected |

No GitHub settings changed during this audit.

## Public DNS and domain findings

This table records the A2-authoritative snapshot taken before the owner moved
the domain's delegation to Cloudflare later on 2026-07-19. Current DNS state is
maintained in `CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md`.

| Capability | Status | Finding |
|---|---|---|
| Authoritative DNS | Available | Four `a2hosting.com` authoritative nameservers |
| Apex address | Available | One IPv4 record; no IPv6 record observed |
| `www` mapping | Available | CNAME to the apex; no IPv6 record observed |
| QuireForge subdomain | Available | One IPv4 record; no CNAME or IPv6 record observed |
| Wildcard DNS | Unavailable | A randomized test name returned `NXDOMAIN` |
| Address-record TTL | Available | Approximately two hours at inspection |
| Mail routing | Available | Existing MX data is present and was not recorded or changed |
| CAA | Unavailable | No CAA record observed; issuance is not DNS-restricted to a named CA |
| DNSSEC delegation | Unavailable | No DS record or authenticated-data response observed |
| Zone Editor | Available | cPanel exposes simple and advanced zone editing; no record changed |
| Redirect configuration | Available | cPanel exposes redirects, but QuireForge currently has no HTTPS or canonical-host redirect |
| Proposed document root | Available | Dedicated root outside `public_html`; account-specific prefix is redacted |

The QuireForge deployment must not alter MX, SPF, DKIM, DMARC, unrelated TXT,
or other domain records. Any future CAA, DNSSEC, address, or redirect change is
a separate state-changing approval.

## Public TLS and HTTP findings

| Capability | Status | Finding |
|---|---|---|
| Main-site certificate | Available | Valid Let's Encrypt certificate covering the apex and wildcard; expires 2026-08-27 |
| QuireForge certificate | Available | Distinct valid Let's Encrypt certificate; expires 2026-10-17 |
| TLS 1.2 | Available | Negotiated successfully on both hosts |
| TLS 1.3 | Available | Negotiated successfully on both hosts |
| TLS 1.0 / 1.1 | Unavailable | Rejected by both hosts |
| Web server | Available | Public response identifies LiteSpeed |
| HTTPS response | Available | Both hosts returned `403 Forbidden`; content access is restricted |
| HTTP-to-HTTPS redirect | Unavailable | HTTP returns 403 directly; no redirect is configured |
| HSTS | Available | `max-age=63072000; includeSubDomains` already present |
| Clickjacking protection | Available | `X-Frame-Options: SAMEORIGIN` observed |
| MIME-sniffing protection | Available | `X-Content-Type-Options: nosniff` observed |
| Content Security Policy | Unavailable on placeholder | Not present on the observed 403 response |
| Referrer / Permissions policy | Unavailable on placeholder | Not present on the observed 403 response |
| Compression and static caching | Unknown | Cannot evaluate representative content through the 403 response |
| Mixed-content state | Unknown | No production page was readable |

HSTS must not be removed or changed during QuireForge work. Because it already
includes subdomains, a broken or expired QuireForge certificate can make the
site unreachable without an HTTP fallback.

## Existing-site findings

The main site and QuireForge subdomain have separate cPanel document roots.
Neither root contains an index or `.htaccess`; the main root contains only a
small public `robots.txt`, while QuireForge contains provider-managed
`.well-known/acme-challenge` and `cgi-bin` directories. No WordPress, Astro,
package manifest, or Git checkout marker was found in either root. The 403 is
therefore the expected no-index placeholder response, not evidence of a failed
QuireForge deployment.

No canonical redirect, custom error page, sitemap, application-level caching,
compression rule, or analytics marker was established. No separate QuireForge
staging hostname/root was found or created. Future synchronization must
preserve `.well-known` and `cgi-bin` rather than using an unqualified
`--delete` against the document root.

No existing-site content was downloaded into the repository.

## Hosting plan and resource matrix

| Item | Status | Finding |
|---|---|---|
| Hosting class | Available | Shared CloudLinux/cPanel environment |
| Plan/tier | Available | A2 `Drive 2020` |
| Domain/subdomain allowance | Available | Reported unlimited; five subdomains in use at audit time |
| Disk quota and use | Available | Unlimited account quota; approximately 4.2 GiB used |
| Inode quota and use | Available | 102,931 of 600,000 used |
| Bandwidth policy/use | Available | Unlimited account quota; approximately 645 MiB used in the current month |
| CPU, RAM, process, entry-process, I/O limits | Available | 100% CPU, 1 GiB RAM, 75 processes, 50 entry processes, 4 MiB/s I/O, 1,024 IOPS |
| Concurrent connection and file-size limits | Unknown | Plan/provider evidence |
| Database allowance | Not relevant initially | Static production site uses no database |
| Backup size/retention limits | Unknown | Backup interface and plan evidence |

## cPanel feature matrix

| Feature | Status | Notes |
|---|---|---|
| Domains / subdomains | Available | Dedicated QuireForge subdomain and isolated root confirmed |
| Zone Editor / redirects | Available | Interfaces enabled; no settings changed |
| File Manager / Disk Usage | Available | Interfaces and account usage API available |
| Backup / Backup Wizard | Available | Feature enabled; no downloadable backups listed and provider retention unknown |
| Git Version Control | Available | Feature and read-only API work; no repositories configured |
| Terminal / SSH Access | Available | Bash over key-authenticated SSH on the provider port |
| SSH key management | Available | Dedicated audit key imported and authorized |
| Cron Jobs | Available | UI feature enabled; this server lacks the current Cron UAPI module |
| Application Manager / Node.js | Provider-disabled / degraded | Passenger disabled; Node selector shown but Node/npm absent from shell |
| SSL/TLS Status / AutoSSL | Available | Let's Encrypt AutoSSL active for QuireForge and its `www` alias |
| ModSecurity / malware tools | Mixed | ModSecurity installed/enabled; account malware scanner disabled |
| Hotlink Protection / Directory Privacy | Available | cPanel features enabled |
| Error Pages | Available | cPanel interface enabled; no QuireForge custom page configured |
| Metrics / raw access / error / resource logs | Available | Interfaces and account log paths present |
| MIME Types / Indexes | Available / untested | Interfaces enabled; representative static artifact not served |
| API tokens / 2FA | Available / not inspected | Interfaces enabled; no token created and account 2FA state not retained |
| IP blocking / WAF | Available | IP blocker and ModSecurity available; 403 was caused by the empty root |
| Server caching / optimization | Available / untested | LiteSpeed and optimize interface present; artifact behavior untested |
| Staging / clone / restore | Unknown | Needed for deployment and rollback recommendation |

cPanel documents that providers may disable Git Version Control, SSH, SSL
Status, and other interfaces through feature management. General cPanel
documentation therefore cannot establish their availability on this account.

## Server and runtime matrix

| Capability | Status | Public or local finding |
|---|---|---|
| Server OS / cPanel version | Available | EL7-derived CloudLinux kernel; cPanel 110.0.136 |
| Web server | Available | LiteSpeed publicly; Apache 2.4.68 reported by cPanel |
| SSH / shell | Available | Nonstandard provider port, Bash, dedicated key, strict pinned Ed25519 host key |
| Git | Available | 2.39.1 |
| Node, npm, Corepack, pnpm | Unavailable in shell | On-host Astro build rejected as an architecture |
| `rsync`, SFTP, SCP, `tar` | Available | `rsync` 3.1.2 and transfer/archive tools present |
| PHP / Python | Available | PHP 8.3.31 and Python 3.6.8; unnecessary for static production |
| Symlink support | Partially available | Account symlink observed; public-root switching not tested |
| Cron | Available / not required | Feature enabled; preferred deployment does not depend on it |
| Background processes | Not relevant | Production is static and requires no service process |
| Custom binaries | Not relevant initially | Build occurs on GitHub-hosted runners |
| cPanel Git hooks / `.cpanel.yml` | Available / untested | Git feature/API works; no repository or deployment test created |
| Outbound GitHub connectivity | Unknown | Needed only for cPanel pull deployment |
| Strict SSH host verification | Available | Enforced with a dedicated pinned known-hosts file; provider confirmation remains desirable |

The local development host is Ubuntu 26.04 LTS on x86_64 with Node 22,
Git/GitHub CLI, `rsync`, OpenSSH, SFTP/SCP, `tar`, DNS, TLS, and HTTP tooling.
Rust, Cargo, pnpm, and Tauri development headers are not installed. Runtime
GTK/WebKitGTK libraries and XDG desktop portals are installed. No packages were
installed during this audit.

## Backup and recovery findings

cPanel Backup/Backup Wizard is enabled, but its read-only API listed no account
backup files. Provider-managed frequency, retention, quota impact, file-level
restore, and on-demand backup support remain unknown.

QuireForge deployment must retain a project-owned release manifest containing
the Git commit, build timestamp, and artifact checksum. Provider backups cannot
replace a versioned deployment rollback. No backup was created during this
read-only phase.

## Security findings

- GitHub secret scanning and push protection are enabled.
- GitHub Actions currently allows all actions and does not require immutable
  action SHAs; the future workflow must pin reviewed actions and reduce policy
  where appropriate.
- The default workflow token is read-only, which is a suitable starting point.
- The default branch is not protected and no deployment environment exists.
- Existing HSTS already covers subdomains.
- TLS 1.2 and 1.3 work; TLS 1.0 and 1.1 do not.
- A dedicated passphrase-protected audit SSH key was created locally and its
  public key authorized; no password, API token, or deployment credential was
  accessed or created.
- ModSecurity is enabled for QuireForge; the account malware scanner is
  disabled. Account directories are owner-restricted and the document roots
  use the provider's expected shared-web-group permissions.
- OpenSSH 7.4 warned that the connection lacks post-quantum key exchange.
- cPanel 110/CloudLinux 7 receives Extended Lifecycle Support only through
  January 1, 2027; provider migration timing is a material risk.

## Deployment-method comparison

| Method | Availability | Security | Reliability / rollback | Complexity | Preliminary disposition |
|---|---|---|---|---|---|
| GitHub Actions build + SSH/`rsync` | Available, deployment untested | Strong when environment-gated with a separate deployment key and pinned host key | Medium–high; atomic switch untested | Medium | Viable but superseded |
| cPanel Git + `.cpanel.yml` | Available, deployment untested | Executes repository deployment tasks on an aging host | Medium; Node build unavailable | Medium–high | Superseded |
| Manual verified artifact upload | Available in principle | Greater human-error risk | Low–medium unless each upload is backed up | Low setup / high recurring effort | Emergency fallback only |

cPanel requires a checked-in top-level `.cpanel.yml`, at least one branch, and
a clean tree for Git deployment. Its documentation explicitly warns against
wildcard copying because doing so can publish `.git` and other unintended data.

## Superseded production architecture

The audit confirmed SSH and `rsync` but did not test release switching. The
following design was viable before Cloudflare Pages superseded it:

1. GitHub Actions checks out an exact approved commit.
2. It installs pinned dependencies from a committed lockfile.
3. It runs type, unit, accessibility, link, and static build checks.
4. It creates a deterministic static artifact and checksum.
5. A protected `production` environment requires explicit approval before its
   dedicated credentials become available.
6. Strict host verification validates a separately confirmed fingerprint.
7. Only generated static output is transferred to a new versioned release.
8. The release is verified before switching the public root or release link.
9. The previous release remains available for immediate rollback.
10. Post-deployment checks confirm HTTPS, canonical host, assets, headers, 404,
    and that the main site is unaffected.

The final layout and switching mechanism depend on the real document root and
symlink policy. No path is assumed here.

## Remaining untested A2 questions

1. Can an A2-hosted release live outside the public root and switch atomically?
2. What staging/clone workflow applies to a plain static site?
3. What provider backups and file-level restore paths cover the document root?
4. Can the account reach GitHub outbound, and what transfer/file-size policies
   apply? These tests are no longer needed for the selected Pages architecture.

## Authenticated access record

A dedicated passphrase-protected QuireForge audit key was authorized after
separate approval. Strict host checking used a locally pinned Ed25519 key after
consistent trust-on-first-use observations; independent A2 confirmation remains
desirable. The private key and passphrase were never displayed or committed.
Only read-only version, quota, feature, filesystem-metadata, and configuration
commands ran. No directory, permission, `.htaccess`, package, content, DNS,
deployment, mail, or database change occurred. The key should be revoked after
the owner approves audit-access cleanup.

## Authoritative sources

- [cPanel SSH Access](https://docs.cpanel.net/cpanel/security/ssh-access/)
- [cPanel Git Version Control](https://docs.cpanel.net/cpanel/files/git-version-control/)
- [cPanel Git deployment guide](https://docs.cpanel.net/knowledge-base/web-services/guide-to-git-deployment/)
- [cPanel SSL/TLS Status](https://docs.cpanel.net/cpanel/security/ssl-tls-status/)
- [cPanel Backup Wizard](https://docs.cpanel.net/cpanel/files/backup-wizard/)
- [cPanel product lifecycle](https://docs.cpanel.net/knowledge-base/cpanel-product/product-versions-and-the-release-process/)
- [GitHub secure-use reference](https://docs.github.com/en/actions/reference/security/secure-use)
- [GitHub deployments and environments](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments)
- Public DNS, HTTP, and TLS observations recorded on 2026-07-19.
