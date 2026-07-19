# A2 Hosting Capability Audit

- Audit date: 2026-07-19
- Production target: `https://quireforge.jamesjennison.net`
- Status: public/local phase complete; authenticated read-only phase pending
- Hosting changes made: none

## Scope and redaction

This report evaluates whether the owner's A2 Hosting cPanel account can safely
host QuireForge's statically generated Astro website. The first phase used only
the local QuireForge checkout, read-only GitHub metadata, public DNS, public
HTTP/TLS responses, and authoritative documentation.

Raw IP addresses, mail-routing values, account usernames, home paths, account
IDs, credentials, and unrelated hosting content are intentionally omitted.
No cPanel login, SSH connection, API token, DNS mutation, upload, or deployment
occurred.

Status labels mean:

- **Available**: directly observed for this account or public endpoint.
- **Unavailable**: directly tested and absent or rejected.
- **Provider-disabled**: an authenticated account view establishes that the
  provider disabled the feature.
- **Requires support**: provider action is required.
- **Unknown**: not safely established yet.
- **Not relevant**: unnecessary for the static QuireForge site.

## Executive summary

The dedicated subdomain is already present in authoritative A2-hosted DNS and
has a distinct, valid Let's Encrypt certificate. Both it and the existing main
site currently return `403 Forbidden` to the audit clients. Public responses
identify LiteSpeed and include a two-year HSTS policy with
`includeSubDomains`. This means HTTPS continuity for the QuireForge subdomain
is already security-sensitive.

The public phase cannot determine the A2 plan, document root, current files,
reason for the 403, account quotas, backup coverage, shell availability, cPanel
feature set, or whether versioned/atomic deployment is possible. Those items
remain blocked on separately approved authenticated read-only access.

Subject to that audit, the preliminary preferred architecture is: GitHub
Actions builds and validates Astro, a protected production environment gates
access to a dedicated SSH key, and `rsync` transfers only the verified static
artifact into a versioned release directory. A symlink or similarly atomic
switch is preferred if supported. This is not a final deployment decision and
does not authorize credential creation or deployment.

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
| Zone Editor | Unknown | Requires authenticated cPanel inspection |
| Redirect configuration | Unknown | Public requests are denied before a canonical redirect is demonstrated |
| Proposed document root | Unknown | Must be read from cPanel; never infer or hard-code it |

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
| HTTP-to-HTTPS redirect | Unknown | HTTP also returned 403, so redirect behavior was not established |
| HSTS | Available | `max-age=63072000; includeSubDomains` already present |
| Clickjacking protection | Available | `X-Frame-Options: SAMEORIGIN` observed |
| MIME-sniffing protection | Available | `X-Content-Type-Options: nosniff` observed |
| Content Security Policy | Unknown | Not present on the observed 403 response |
| Referrer / Permissions policy | Unknown | Not present on the observed 403 response |
| Compression and static caching | Unknown | Cannot evaluate representative content through the 403 response |
| Mixed-content state | Unknown | No production page was readable |

HSTS must not be removed or changed during QuireForge work. Because it already
includes subdomains, a broken or expired QuireForge certificate can make the
site unreachable without an HTTP fallback.

## Existing-site findings

The main site and QuireForge subdomain denied both ordinary command-line and
browser-style audit requests. Search discovery returned no indexed pages. The
main site's `robots.txt` was public and specified a crawl delay; common sitemap
and security-contact paths were not found.

The following remain unknown and require authenticated inspection or
owner-provided sanitized information:

- main-site platform and private document root;
- current QuireForge document root and contents;
- `.htaccess`, redirect, error-page, caching, compression, and security rules;
- analytics and deployment mechanism;
- whether the 403 is intentional, permission-related, or placeholder behavior;
- whether a separate staging hostname/root already exists.

No existing-site content was downloaded into the repository.

## Hosting plan and resource matrix

| Item | Status | Required evidence |
|---|---|---|
| Shared / managed VPS / unmanaged VPS / dedicated | Unknown | cPanel/account plan view or owner confirmation |
| Plan/tier | Unknown | Sanitized plan name only |
| Domain/subdomain allowance | Unknown | Account limits; existing subdomain proves at least one is configured |
| Disk quota and use | Unknown | Sanitized cPanel Disk Usage/quota output |
| Inode quota and use | Unknown | Sanitized Resource Usage or provider confirmation |
| Bandwidth policy/use | Unknown | Sanitized Metrics/plan output |
| CPU, RAM, process, entry-process, I/O limits | Unknown | Sanitized Resource Usage/CloudLinux view |
| Concurrent connection and file-size limits | Unknown | Plan/provider evidence |
| Database allowance | Not relevant initially | Static production site uses no database |
| Backup size/retention limits | Unknown | Backup interface and plan evidence |

## cPanel feature matrix

| Feature | Status | Notes |
|---|---|---|
| Domains / subdomains | Unknown | Public DNS and TLS exist; management UI not audited |
| Zone Editor / redirects | Unknown | Authenticated inspection required |
| File Manager / Disk Usage | Unknown | Authenticated inspection required |
| Backup / Backup Wizard | Unknown | Feature and provider retention must be verified |
| Git Version Control | Unknown | Provider may disable it; shell access affects functionality |
| Terminal / SSH Access | Unknown | cPanel supports these generally; account entitlement unknown |
| SSH key management | Unknown | Preferred access/deployment method if account exposes it |
| Cron Jobs | Unknown | Not required for preferred push deployment |
| Application Manager / Node.js | Unknown | Node is build-only if available; no persistent app is needed |
| SSL/TLS Status / AutoSSL | Unknown | Public certificates exist; renewal/status UI not inspected |
| ModSecurity / malware tools | Unknown | Must not be disabled for deployment convenience |
| Hotlink Protection / Directory Privacy | Unknown | Evaluate compatibility without weakening controls |
| Error Pages | Unknown | Needed for an intentional static 404 experience |
| Metrics / raw access / error / resource logs | Unknown | Audit only QuireForge-relevant views |
| MIME Types / Indexes | Unknown | Static asset types and directory listing policy require checks |
| API tokens / 2FA | Unknown | Token creation is not authorized; 2FA state should be confirmed |
| IP blocking / WAF | Unknown | May explain the observed 403 |
| Server caching / optimization | Unknown | Must not rewrite immutable Astro assets incorrectly |
| Staging / clone / restore | Unknown | Needed for deployment and rollback recommendation |

cPanel documents that providers may disable Git Version Control, SSH, SSL
Status, and other interfaces through feature management. General cPanel
documentation therefore cannot establish their availability on this account.

## Server and runtime matrix

| Capability | Status | Public or local finding |
|---|---|---|
| Server OS / cPanel version | Unknown | Authenticated inspection required |
| Web server | Available | LiteSpeed observed publicly |
| SSH hostname / port / shell type | Unknown | Must be explicitly confirmed before connection |
| Git, Node, npm, Corepack, pnpm | Unknown | Server-side build cannot be assumed |
| `rsync`, SFTP, SCP, `tar` | Unknown | Required to choose transfer/rollback method |
| Symlink support | Unknown | Determines atomic-release feasibility |
| Cron | Unknown / not required | Preferred deployment does not depend on it |
| Background processes | Not relevant | Production is static and requires no service process |
| Custom binaries | Not relevant initially | Build occurs on GitHub-hosted runners |
| cPanel Git hooks / `.cpanel.yml` | Unknown | Account-specific feature audit required |
| Outbound GitHub connectivity | Unknown | Needed only for cPanel pull deployment |
| Strict SSH host verification | Unknown | Required for any SSH-based method |

The local development host is Ubuntu 26.04 LTS on x86_64 with Node 22,
Git/GitHub CLI, `rsync`, OpenSSH, SFTP/SCP, `tar`, DNS, TLS, and HTTP tooling.
Rust, Cargo, pnpm, and Tauri development headers are not installed. Runtime
GTK/WebKitGTK libraries and XDG desktop portals are installed. No packages were
installed during this audit.

## Backup and recovery findings

cPanel's Backup Wizard can expose full and partial backups when enabled, but
full restore may require provider/WHM assistance. Account availability,
provider-managed backup frequency, retention, quota impact, file-level restore,
and on-demand backup support remain unknown.

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
- No cPanel, SSH, API, or deployment credential was accessed or created.
- Two-factor authentication, ModSecurity, malware scanning, account shell
  isolation, and file permissions remain unknown.

## Deployment-method comparison

| Method | Availability | Security | Reliability / rollback | Complexity | Preliminary disposition |
|---|---|---|---|---|---|
| GitHub Actions build + SSH/`rsync` | Unknown until SSH audit | Strong when environment-gated with a dedicated key and pinned host key | High if versioned releases and atomic switch are supported | Medium | **Preferred, conditional** |
| cPanel Git + `.cpanel.yml` | Unknown until feature audit | Keeps Git pull server-side but executes repository deployment tasks | Medium; clean tree and explicit deployment commands required | Medium–high | Secondary option |
| Manual verified artifact upload | Likely possible but unverified | No CI credential; greater human-error risk | Low–medium unless each upload is backed up | Low setup / high recurring effort | Emergency fallback |

cPanel requires a checked-in top-level `.cpanel.yml`, at least one branch, and
a clean tree for Git deployment. Its documentation explicitly warns against
wildcard copying because doing so can publish `.git` and other unintended data.

## Preliminary production architecture

If authenticated inspection confirms SSH, `rsync`, and safe release switching:

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

## Remaining authenticated audit questions

1. What exact A2 plan and cPanel version serve the domain?
2. What SSH hostname, port, shell mode, and host-key fingerprints apply?
3. What is the exact QuireForge document root, and what currently causes 403?
4. Are SSH key management, Terminal, Git Version Control, backups, AutoSSL,
   ModSecurity, metrics, cron, and API tokens exposed?
5. What are the account's disk, inode, bandwidth, CPU, memory, process, I/O,
   connection, and file-size limits?
6. Are Git, Node, npm/Corepack, `rsync`, SFTP/SCP, `tar`, and symlinks available?
7. Can releases live outside the public root and switch atomically?
8. Is a staging subdomain/root available without affecting the main site?
9. What provider backups and file-level restore paths cover the document root?
10. Why are public requests receiving 403, and which security control owns it?

## Proposed authenticated access boundary

Use a dedicated QuireForge SSH key, authorized through cPanel, only after a
separate approval naming the exact hostname, port, account scope, and verified
host-key fingerprint. The private key stays in an approved local credential
store and is never pasted into chat or committed. The initial authenticated
session runs read-only version, quota, feature, filesystem-metadata, and
configuration-inspection commands only. It does not create directories, alter
permissions, edit `.htaccess`, install packages, run deployments, or inspect
unrelated sites, mail, or databases.

## Authoritative sources

- [cPanel SSH Access](https://docs.cpanel.net/cpanel/security/ssh-access/)
- [cPanel Git Version Control](https://docs.cpanel.net/cpanel/files/git-version-control/)
- [cPanel Git deployment guide](https://docs.cpanel.net/knowledge-base/web-services/guide-to-git-deployment/)
- [cPanel SSL/TLS Status](https://docs.cpanel.net/cpanel/security/ssl-tls-status/)
- [cPanel Backup Wizard](https://docs.cpanel.net/cpanel/files/backup-wizard/)
- [GitHub secure-use reference](https://docs.github.com/en/actions/reference/security/secure-use)
- [GitHub deployments and environments](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments)
- Public DNS, HTTP, and TLS observations recorded on 2026-07-19.
