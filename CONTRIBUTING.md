# Contributing to QuireForge

Thank you for helping build QuireForge. The project is early in development;
many documented features are planned rather than implemented.

QuireForge is an unofficial community project. Contributors must not imply
OpenAI endorsement or use OpenAI, ChatGPT, or Codex assets in ways that suggest
official ownership.

## Before starting

1. Read [the roadmap](docs/ROADMAP.md),
   [architecture](docs/ARCHITECTURE.md), and applicable decisions under
   `docs/DECISIONS/`.
2. Search existing issues before proposing substantial work.
3. Open an issue for architectural, security-sensitive, integration, storage,
   packaging, or externally visible changes before investing heavily.
4. Never include secrets, personal Codex data, private integration metadata, or
   unredacted diagnostics in an issue, fixture, commit, or pull request.

## Local checks

The repository uses Python, Node/pnpm, and Rust/Tauri quality gates:

```bash
python3 scripts/validate_repository.py
pnpm validate
pnpm test:e2e
git diff --check
git status --short --branch
```

Install the website and desktop prerequisites from [Building](docs/BUILDING.md)
first. Run every check documented for the subsystem you change; browser and
native launch checks remain separate from the non-interactive validation suite.

## Pull requests

- Create a focused branch from the current default branch.
- Keep commits understandable and avoid unrelated formatting churn.
- Add tests for behavior changes and deterministic fixtures for Codex or
  integration protocols.
- Update documentation and the changelog when behavior changes.
- Preserve user directories, Git state, sessions, and existing local changes.
- Describe security, privacy, accessibility, and compatibility effects.
- Confirm the contribution uses only supported public interfaces and assets you
  have the right to contribute.

By submitting a contribution, you agree that it may be distributed under the
license included in this repository.

Maintainers may ask to split broad changes, revise unsupported claims, or add
tests before review. A passing automated check does not replace architectural,
security, or accessibility review.

## Integration contributions

Do not add private ChatGPT endpoints, browser-token extraction, scraped
catalogs, invented integration identifiers, or code that bypasses account,
workspace, regional, plan, or administrator controls. Sanitize protocol
fixtures and classify behavior as stable official, experimental official,
local QuireForge functionality, or unsupported.

## Community expectations

Participation is governed by [the Code of Conduct](CODE_OF_CONDUCT.md).
Security reports follow [the security policy](SECURITY.md), not public issues.
General help and project-status expectations are described in
[the support guide](SUPPORT.md).
