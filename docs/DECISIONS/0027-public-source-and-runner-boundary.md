# ADR 0027: Public Source and Persistent Runner Boundary

- Status: Accepted
- Date: 2026-07-23
- Decision owners: Project owner and maintainers
- Amends: [ADR 0003](0003-permanent-quireforge-identity.md) and
  [ADR 0024](0024-webuzo-static-website-hosting.md)

## Context

QuireForge needs public GitHub source visibility so its first beta can receive
GitHub artifact attestations on the current organization plan and so users can
inspect the source and release provenance. The organization already operates
persistent self-hosted Linux runners shared by a selected set of repositories.
A public repository must not allow untrusted fork code to reach those machines.

A dedicated full-history disclosure audit found no credentials or secrets. The
owner deliberately accepted the documented low-sensitivity historical identity,
path, and runner-log disclosures without rewriting Git history or deleting
workflow evidence.

## Decision

- `James-Jennison/quireforge` is a public Apache-2.0 source repository.
- The owner-hosted Webuzo origin remains authoritative for the project website
  and primary package downloads. GitHub Pages and Cloudflare Pages remain
  disabled.
- GitHub prereleases are public source/provenance and secondary artifact
  records; website download activation remains a separate same-origin hosting
  operation.
- The existing organization runner group remains selected-repository scoped
  and explicitly permits public repositories so QuireForge can retain its
  current runners.
- A `pull_request` job using `self-hosted` must execute only when the pull
  request head repository is exactly the base repository.
- `pull_request_target` must never execute repository code and is prohibited by
  the repository validator.
- Fork-origin pull requests do not receive the persistent self-hosted checks.
  Maintainers may add isolated GitHub-hosted fork validation later, but that
  does not weaken this boundary.

## Consequences

- Source, issues, pull requests, Actions logs, and GitHub release records are
  publicly readable.
- Public forks and clones cannot be recalled by a later visibility change.
- Contributors can inspect and propose changes under the existing contribution
  and security policies.
- Persistent runners continue serving trusted QuireForge branches and
  same-repository pull requests, while fork-origin code remains excluded.
- Enabling public-repository access for the selected runner group does not make
  other selected private repositories public.
- Provider identifiers, runner registration tokens, credentials, private
  diagnostics, and hosting details remain excluded from source.
