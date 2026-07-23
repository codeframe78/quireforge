# Webuzo Deployment

Status: production active through a Webuzo-managed static origin and Cloudflare
DNS/proxy edge; canonical origin TLS renewal is provider-managed.

## Production contract

- Public URL: `https://quireforge.jamesjennison.net`.
- Origin: Webuzo-managed Apache serving static Astro output.
- Edge: Cloudflare authoritative DNS, proxy, cache, and public TLS.
- Runtime services and database: none.
- Source, Git metadata, credentials, internal documentation, and provider
  diagnostics: excluded from the public artifact.
- Cloudflare Pages and GitHub Pages: disabled and not fallback hosts.

Private provider account names, server addresses, document-root paths,
certificate identifiers, DNS record identifiers, and backup identifiers must
be resolved at execution time and must not be committed.

## Build contract

From the repository root:

```bash
pnpm install --frozen-lockfile
pnpm validate
pnpm --filter @quireforge/website build
```

The reviewed artifact is `apps/website/dist/` only. It must be built outside
public storage and must not contain source trees, `.git`, `.env`, credentials,
keys, certificate material, private documentation, dependency directories,
logs, source maps, or generated support bundles.

## Promotion contract

Every staging or production promotion requires separate owner approval and
must:

1. resolve and confirm the exact provider-managed hostname and isolated
   destination without recording private identifiers;
2. capture a recoverable pre-change state;
3. validate the source revision and complete static artifact outside public
   storage;
4. record an ephemeral per-file manifest outside the repository;
5. stage and promote only that artifact with explicit ownership and
   permissions;
6. leave provider-generated virtual-host and global server configuration
   untouched;
7. verify server syntax, TLS, routes, redirects, the custom 404, headers,
   canonical metadata, accessibility, and unaffected sites; and
8. capture and rehearse a recoverable post-change state before cleanup.

The version-controlled `.htaccess` owns only domain-scoped application
behavior. It preserves provider ACME paths, refuses noncanonical hosts,
enforces HTTPS, applies reviewed security headers, and configures immutable
hashed-asset caching.

## Rollback

Application rollback restores the retained prior artifact through the provider
and reruns the origin and edge checks. DNS, certificate, domain, or
document-root rollback is a distinct owner-approved operation and must target
only identifiers freshly resolved for the canonical QuireForge hostname.

Acceptance outcomes are recorded in the Milestone 16 reports. Operational
manifests, account details, paths, backup records, and private diagnostics stay
outside source control.
