# cPanel Deployment

Status: preliminary architecture; no credential, hosting setting, staging
deployment, or production deployment is authorized.

## Production target

- Public URL: `https://quireforge.jamesjennison.net`.
- Host: the owner's A2 Hosting cPanel account.
- Source and CI: `codeframe78/quireforge` on GitHub.
- Release binaries: GitHub Releases.
- Website output: static Astro files only.
- Document root: deliberately unknown until authenticated read-only audit.

GitHub Pages is not a production or fallback host. It remains disabled.

## Preferred deployment shape

The preferred option, conditional on the capability audit, is a GitHub Actions
build followed by SSH/`rsync` delivery of a verified static artifact.

The workflow must:

1. Build and validate without deployment secrets on pull requests.
2. Pin the Node/package-manager versions and install from a lockfile.
3. Pin reviewed GitHub Actions to immutable commit SHAs.
4. Generate the Astro site for the production origin and root base path.
5. Run accessibility, broken-link, asset, 404, and secret checks.
6. Record the source commit and artifact checksum.
7. Enter a protected `production` environment only after successful checks and
   explicit approval.
8. Use a dedicated deployment key and strict host-key verification.
9. Transfer only generated output, never source, `.git`, local configuration,
   or secrets.
10. Verify a versioned release before switching it live.
11. Preserve the previous release until live verification finishes.

Concurrency must prevent overlapping production deployments. A failed build or
failed staging verification must not alter the current public release.

## Credential contract

Potential GitHub environment secret names are documentation-only placeholders:

- `A2_SSH_HOST`
- `A2_SSH_PORT`
- `A2_SSH_USERNAME`
- `A2_SSH_PRIVATE_KEY`
- `A2_SSH_HOST_FINGERPRINT`
- `A2_DEPLOY_PATH`

No value may appear in Git, logs, generated artifacts, pull-request jobs, or
chat. The key must be dedicated to QuireForge, narrowly scoped where the host
supports it, and independently revocable. Password SSH and plain FTP are
prohibited.

## Filesystem layout

Do not hard-code a cPanel home directory or document root. If supported, use:

```text
non-public release root/
  releases/<commit>/
  current -> releases/<approved-commit>/

dedicated QuireForge document root -> current
```

If symlink switching is unavailable, deploy to a staging directory, back up the
current public artifact, synchronize an explicit allowlist, and avoid broad
deletion. Source and `.git` stay outside public storage.

## Pre-deployment plan

Before every state-changing deployment, show and obtain approval for:

- destination hostname and exact document root;
- source commit and artifact checksum;
- proposed release directory;
- files to add, replace, and remove;
- credential and host-key identity;
- backup/release currently available for rollback;
- staging results and post-deployment checks;
- exact rollback operation.

Approval to edit source does not authorize staging or production deployment.

## Verification

Staging and production checks must cover:

- DNS and valid HTTPS for the canonical host;
- HTTP redirect behavior;
- HSTS continuity and absence of mixed content;
- HTML, JavaScript, CSS, fonts, images, favicon, and download links;
- canonical and social metadata;
- robots, sitemap, and custom 404 behavior;
- CSP and other security headers;
- compressed content and immutable-asset cache behavior;
- desktop and mobile accessibility/Lighthouse targets;
- continued availability of `www.jamesjennison.net`.

## Rollback

Rollback switches to the previously verified release when atomic switching is
supported. Otherwise it restores the pre-deployment backup with an explicit
file manifest. After rollback, rerun the same public checks and record the
restored commit and checksum.

Never delete the previous release merely because the new upload completed.

## Alternative cPanel Git deployment

cPanel Git remains a secondary option pending account verification. It requires
a clean cPanel-managed checkout, at least one branch, and a checked-in
`.cpanel.yml`. Deployment tasks must copy an explicit generated directory; they
must never wildcard-copy a repository into the public root. Server-side Node
builds are unacceptable unless versions, resource limits, reproducibility, and
failure isolation are demonstrated.

See [the capability audit](A2-HOSTING-CAPABILITY-AUDIT.md) for evidence,
unknowns, and the method comparison.
