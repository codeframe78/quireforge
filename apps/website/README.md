# QuireForge Website

This package contains the statically generated Astro website for QuireForge.
It has no server runtime, database, analytics, public repository integration,
or production credentials.

## Local commands

From the repository root:

```bash
pnpm install --frozen-lockfile
pnpm dev
pnpm validate
pnpm test:e2e
```

The pinned package manager is pnpm 11.15.0. If an older distribution Corepack
cannot launch it, use `npx --yes pnpm@11.15.0` in place of `pnpm`; this does not
change the repository contract.

The production site is `https://quireforge.jamesjennison.net`. The build output
in `dist/` is a Webuzo-compatible static artifact deployed only to the exact
QuireForge document root reported by Webuzo. Its scoped Apache `.htaccess`
enforces the canonical host, HTTPS, security headers, and the reviewed
domain-specific HSTS policy.

Future Webuzo domain or document-root changes, origin SSL operations,
Cloudflare changes, and deployments remain approval-gated operations. The site
intentionally uses no Astro server adapter or background application process.
