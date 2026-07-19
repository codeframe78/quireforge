# QuireForge Website

This package contains the statically generated Astro website for QuireForge.
It has no server runtime, database, analytics, or production credentials.

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

The production origin is `https://quireforge.jamesjennison.net`. Cloudflare
Pages project creation, Git integration, custom-domain activation, DNS changes,
and deployment are separate approval-gated operations.

Expected future Cloudflare Pages settings:

- Root directory: `apps/website`
- Build command: `pnpm build`
- Output directory: `dist`
- Production branch: `main`

The site is intentionally static and does not use the Astro Cloudflare adapter.
