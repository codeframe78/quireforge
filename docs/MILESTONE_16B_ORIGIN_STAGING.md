# Milestone 16B: Webuzo Origin-Only Staging

- Status: Complete
- Date: 2026-07-22
- Public DNS changes during 16B: None

## Outcome

After separate owner approval, the reviewed static artifact was staged on the
Webuzo-managed Apache origin without activating the public hostname. Webuzo
owned the isolated document root, generated server configuration, origin TLS,
ownership, backup, and rollback boundaries.

No generated virtual-host or global server file was hand-edited. No public DNS,
unrelated hostname, GitHub setting, persistent process, reverse proxy,
database, or application port changed.

## Acceptance evidence

- The provider-reported destination was confirmed as isolated before promotion.
- The artifact was built outside public storage and contained only reviewed
  static output.
- File ownership and permissions were checked without recording account names
  or server paths in the repository.
- Apache configuration, trusted origin TLS, redirects, the custom 404, security
  headers, canonical metadata, and every sitemap route passed.
- Pre-change and post-deployment recovery points were verified.
- A restoration rehearsal reproduced the staged artifact without affecting
  unrelated sites.
- Temporary deployment, certificate, and restore material was removed.

The repository intentionally omits provider account identifiers, document-root
paths, server addresses, certificate details, backup identifiers, and artifact
checksums.

## Rollback

Before public activation, content rollback restores the retained empty or prior
document-root state through the hosting provider, then revalidates the server
configuration and unaffected sites. Domain and TLS rollback remain separate
owner-approved provider operations.

Milestone 16C subsequently activated the canonical public hostname.
