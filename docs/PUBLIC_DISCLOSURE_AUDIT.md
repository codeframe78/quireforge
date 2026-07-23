# Public Disclosure Audit

- Audit date: 2026-07-23
- Scope: complete reachable Git history, GitHub collaboration metadata,
  Actions history, release artifacts, repository settings, and organization
  self-hosted runner policy
- Decision: accepted for deliberate public-source publication

## Scope and method

The audit fetched every remote branch, tag, and pull-request head available to
the repository and inspected all reachable objects rather than only the
current tree. The reviewed graph contained 144 commits, 2,307 objects, and
1,447 unique blobs. Checks included:

- `git fsck --full --no-dangling`;
- Gitleaks 8.30.1, downloaded from its official GitHub release and verified
  against the published checksum before scanning `--all --full-history`;
- independent high-confidence credential, private-key, URL-authentication,
  public-IP, risky-filename, oversized-blob, commit-identity, hostname, and
  local-path searches;
- pull-request, issue, comment, review, workflow-run, job-log, artifact,
  release, environment, deploy-key, secret, and variable inventory; and
- extraction and scanning of the exact Linux artifact produced for the
  approved release-candidate source.

## Findings

No credential, access token, API key, private key, password-bearing URL,
private network address, private server hostname, environment file, credential
database, or generated support bundle was found. The release artifact contained
no QuireForge-owned secret or local user path. Apparent matches were reviewed
as sanitized fixtures, documentation examples, package-version strings, icon
names, or upstream shared-library text.

The owner explicitly accepted these low-sensitivity historical disclosures:

- one personal mailbox in immutable Git author/tag metadata;
- historical local usernames and filesystem paths in earlier documentation;
  and
- generic runner, runner-group, machine, and workspace names in twelve
  retained Actions logs.

Pull request 11 contained two unnecessary local home-directory examples. Its
body was sanitized to use `/home/tester` before publication. No Git history or
workflow run was deleted or rewritten.

## Public runner boundary

The persistent organization runners remain selected for QuireForge's trusted
repository checks. Every self-hosted job triggered by `pull_request` must keep
the same-repository head guard:

```text
github.event_name != 'pull_request' ||
github.event.pull_request.head.repo.full_name == github.repository
```

Fork-origin pull requests therefore cannot execute repository code on those
runners. `pull_request_target` is prohibited. The repository validator enforces
both requirements. The organization runner group must remain selected-repository
scoped and explicitly allow public repositories before QuireForge changes
visibility.

## Publication consequences

Public visibility makes the complete accepted history, collaboration metadata,
Actions logs, and future GitHub release records anonymously readable. A later
visibility reversal cannot recall clones or public forks. QuireForge's
owner-hosted website remains the primary package-download origin; making the
source repository public does not authorize a website deployment or server
upload.
