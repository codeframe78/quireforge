# ADR 0007: Own Project Metadata in Native SQLite

- Status: Accepted and implemented in Milestone 6
- Date: 2026-07-19
- Decision owners: Project maintainers

## Context

QuireForge needs durable project and directory-association metadata without
copying source trees or treating Codex configuration, authentication, or
sessions as application-owned data. Directory attachment is security-sensitive:
the selected path can be moved, replaced, remounted, or retargeted through a
symbolic link between launches.

The metadata workload is local, relational, small, and transaction-oriented.
It needs deterministic migrations, foreign-key enforcement, recovery after a
failed write, and temporary in-memory/file databases for routine tests.

## Decision

- Use `rusqlite` only in the Rust application core. React receives normalized
  typed snapshots and never a general SQL or filesystem command.
- Build SQLite with the reviewed Rust dependency so development and packaged
  behavior do not depend on an unverified host SQLite version.
- Keep ordered SQL migrations embedded in the application, applied in one
  immediate transaction, and recorded in `schema_migrations`.
- Refuse an unknown newer schema instead of modifying or recreating it.
- Store exact selected and last-verified resolved paths plus advisory local
  filesystem and Git/worktree identity evidence.
- Use UUIDv7 project and association identifiers; paths are never database keys.
- Represent detach, relink, and archive as metadata operations. Expose no source
  directory deletion through this lifecycle.
- Keep all routine migration and directory tests in temporary or in-memory
  databases. Tests must not require real Codex sessions or authentication.

## Consequences

- QuireForge can fail closed before future Codex turns without asking React to
  interpret raw paths or identity evidence.
- SQLite writes remain serialized initially. This is appropriate for the small
  local workload and avoids premature connection-pool concurrency.
- Embedded SQLite adds native compile time and binary size but removes a host
  runtime-version dependency.
- Migration history is append-only. Changing an applied migration requires a
  new migration rather than rewriting the earlier SQL.
- Application data can be backed up independently and never contains Codex or
  connector credentials.
