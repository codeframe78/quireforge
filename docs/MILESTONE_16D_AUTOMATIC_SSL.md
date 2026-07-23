# Milestone 16D: Automatic Origin TLS

- Status: Complete
- Date: 2026-07-22
- Scope: Canonical QuireForge origin certificate and renewal only

## Outcome

After separate owner approval, the canonical QuireForge hostname was enrolled
in Webuzo-managed automatic certificate issuance and renewal. No DNS,
Cloudflare, document-root, application, alias, mail, analytics, source
publication, or unrelated certificate setting changed.

## Acceptance evidence

- The Webuzo-supported domain-scoped enrollment path was used.
- The installed origin certificate is trusted, covers only the canonical
  QuireForge hostname, and passes direct-origin validation.
- Cloudflare Full (Strict) continues to validate the origin.
- Provider-managed renewal state and its existing scheduled renewal mechanism
  were verified.
- Public routes, HSTS, the content security policy, and unrelated public sites
  remained healthy.
- Pre-enrollment and post-enrollment recovery points passed integrity and
  restoration checks.
- Temporary challenge and restore material was removed.

Exact certificate dates, serials, fingerprints, provider account identifiers,
server paths, backup identifiers, and renewal scheduling details are
operational secrets and are not stored in the repository.

## Rollback

Certificate rollback restores the prior trusted provider-managed state through
the supported hosting interface, validates the server configuration, and
repeats direct-origin and public Full (Strict) checks. Revocation or changes to
other hostnames require separate approval.
