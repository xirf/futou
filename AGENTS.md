# AGENTS.md — futou Project

## Required Reading

- [`llm/SYSTEM_MAP.md`](llm/SYSTEM_MAP.md): architecture compass. Map-first navigation.
- [`llm/lessons.md`](llm/lessons.md): patterns from prior user corrections.

## Workflow Orchestration

### 1. Plan Mode Default

**MUST call `EnterPlanMode` tool (the actual tool, not just chat text) BEFORE any Write/Edit when ANY trigger fires:**
- Task touches 3+ files
- New crate dependency, new trait, new module, or new public struct/enum
- Refactor of existing ownership model / concurrency flow
- Architectural decision (data source, schema, async runtime, FFI boundary)
- User uses words like "redesign", "rework", "migrate", or "bikin <feature>"

**Skip plan mode for simple work**, strict rules on small fixes create friction:
- Single-line / few-line fix in 1 file
- Typo, rename in known location, format fix
- User lists concrete action items and asks to execute
- User explicitly says "langsung aja" / "skip plan" / equivalent

Once in plan mode: explore, design, write plan to `llm/plans/<scope>-<action>.md`, then `ExitPlanMode` for approval. If something goes sideways during implementation, STOP and re-plan.

### 2. Subagent Strategy
- Use subagents liberally to keep main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One task per subagent for focused execution

### 2.1 Use Related Skills (IMPORTANT)
- **Before acting**, scan the available skills list for one that matches the task. If a skill matches, invoke it via the `Skill` tool, do NOT improvise the same workflow by hand.
- Specifically for this project, prefer:
  - `grilling`, when planning anything architectural / multi-decision before writing code.
  - `rust-add-unit-test` / `rust-add-integration-test`, when shipping new modules or public APIs.
  - `rust-fix-borrow-checker-errors`, on lifetime / borrow-checker errors.
  - `rust-apply-architecture-best-practices`, when scaffolding new crates or modules.
  - `diagnosing-bugs`, when the user reports a bug / regression / perf issue.
  - `review` / `security-review`, before declaring a branch ready for PR.
  - `resolving-merge-conflicts`, on in-progress merge/rebase conflicts.
  - `tdd`, for new logic that needs tests-first.
- Skills are pre-packaged workflows tuned for this repo. Skipping them means duplicating work plus missing repo-specific guardrails.
- If no skill fits, proceed normally. Never invoke a skill that isn't in the available-skills list.

### 3. Self-Improvement Loop
- After ANY correction from the user: update `llm/lessons.md` with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project

### 4. Verification Before Done
- Never mark a task complete without proving it works
- Diff behavior between main and your changes when relevant
- Ask yourself: "Would a staff engineer approve this?"
- Run tests, check logs, demonstrate correctness
- For Rust: run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` on touched crates; run `cargo test` for affected packages; for CLI/service changes, exercise the binary locally

### 5. Demand Elegance (Balanced)
- For non-trivial changes: pause and ask "is there a more elegant way?"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
- Skip this for simple, obvious fixes, don't over-engineer
- Challenge your own work before presenting it

### 6. Autonomous Bug Fixing
- When given a bug report: just fix it. Don't ask for hand-holding
- Point at logs, errors, failing tests, then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how

## Task Management
1. **Plan First**: Write plan to `llm/plans/<scope>-<action>.md` (or in-conversation TaskCreate) with checkable items
2. **Verify Plan**: Check in before starting implementation when scope is non-trivial
3. **Track Progress**: Mark items complete as you go
4. **Explain Changes**: High-level summary at each step
5. **Document Results**: Add work-log section to the plan file (same `llm/plans/<scope>-<action>.md`)
6. **Capture Lessons**: Update `llm/lessons.md` after corrections

### Plan file naming convention
- Location: `llm/plans/`
- Format: `<scope>-<action>.md`, kebab-case, lowercase. No `_` or spaces.
- `<scope>` = the crate / module / feature targeted (e.g. `auth-token-refresh`, `db-pool-config`, `api-router-middleware`).
- `<action>` = type of plan: `cleanup`, `refactor`, `migration`, `extraction`, `plan` (only when the others don't fit).
- Examples: `auth-token-refresh-cleanup.md`, `db-pool-config-plan.md`, `api-router-middleware-migration.md`.

## Core Principles
- **Simplicity First**: Make every change as simple as possible. Impact minimal code.
- **No Laziness**: Find root causes. No temporary fixes. Senior developer standards.
- **Minimal Impact**: Changes should only touch what's necessary. Avoid introducing bugs.

## Navigation & Context Rules

1. **Map-first**: Read SYSTEM_MAP.md at session start. No blind scanning.
2. **Stale map**: SYSTEM_MAP.md missing or out-of-date, update before deeper analysis.
3. **Trace flow** (Rust paradigm):
   `Entry point (main / handler) → Service / Domain layer → Repository / Trait abstraction → Data source (DB pool, HTTP client, filesystem)`. Map foreign patterns to nearest equivalent. Note async runtime boundaries (Tokio tasks, spawn_blocking) and ownership transfer points (Arc, channels, mutexes).
4. **Navigate via rustdoc + SYSTEM_MAP.md + IDE find-references first.** Reach for grep / `find` only when those miss.
5. **Exclude from search**: `target`, `Cargo.lock` (unless dependency work), `.git`, `.vscode`, `.idea`, generated `*.rs` under `build.rs` output dirs, `*_generated.rs`.
6. **Lazy reads**: target the symbol, not the file. Files >500 LOC → read the relevant impl block / function only. Don't load full module files when after one function's logic.
7. **Pre-edit trace**: 1-2 sentence note before any edit: target file plus function flow being touched.
8. **Modularity**: small single-responsibility modules. Business logic separate from I/O and transport layers. No monster `impl` blocks.

### Project doc layout

- `llm/SYSTEM_MAP.md` : architecture compass.
- `llm/codingstyle.md` : coding style rules. Read at session start; conform all new/edited code to it.
- `llm/roadmap.md` : refactor roadmap, if applicable.
- `llm/problem_list.md`: top-level backlog (impact times risk ranked).
- `llm/lessons.md`: patterns plus rules learned from user corrections (review at session start).
- `llm/plans/<scope>-<action>.md`: per-feature/crate plan plus work log plus onboarding (see Task Management for naming convention). Examples: `llm/plans/auth-token-refresh-cleanup.md`, `llm/plans/db-pool-config-plan.md`.
- `llm/<YYYY-MM-DD>-changes.md`: daily change snapshots when scope warrants.

## Documentation (MANDATORY)

### Rustdoc Documentation

Use `///` Rustdoc on each public symbol. IDE tooltip picks it up, refactor-safe.

**Minimum bar:**
- Every public struct / enum / trait / top-level function gets a `///` doc.
- First line = TL;DR purpose (one sentence).
- Below TL;DR (only when non-obvious): invariants, panics, safety notes (for `unsafe` blocks), error conditions. Skip when the code is self-evident.
- Use `[SymbolName]` cross-references, they become clickable in IDE/docs.rs.
- Mark sharp edges with `#[deprecated(note = "...")]`, `#[doc(hidden)]`, etc.
- Every `unsafe fn` or `unsafe` block needs a `# Safety` section explaining the invariant the caller must uphold.

**What NOT to write:**
- Multi-paragraph prose for trivial getters/setters.
- Restating the symbol name in the doc.
- Lists that go stale on refactor (e.g. enumerated callers).

**Example:**
```rust
/// Starts a multi-instance Unix domain socket server.
///
/// * `ctx` - Shared application context for state and shutdown tracking.
/// * `socket_path` - Filesystem path for the socket (e.g., `"/tmp/my_socket"`).
///
/// # Errors
/// Returns an error if the socket path is already bound or unwritable.
///
/// # Note
/// Runs until a shutdown signal is received on the provided channel.
pub async fn run_socket_server(ctx: Arc<AppContext>, socket_path: &str) -> anyhow::Result<()> {
    // ... implementation ...
}
```

### Inline Comments

Default: write none. Let names plus structure tell the story.

Only add an inline `//` comment when the WHY is non-obvious, a hidden constraint, a workaround for a specific bug, behavior that would surprise a reader, or a justification for an `unsafe` block. One short line max; never a multi-paragraph explanation.

**Avoid:**
- Restating what the next line does (`// loop over users` above `for u in &users`).
- "Why we wrote this" prose that belongs in the PR description.
- Step-by-step narration ("Step 1: ...", "Then we ...").
- Comments that go stale on refactor (line numbers, named callers, "see X above").

If you find yourself writing more than one line of `//` to explain a block, extract a named helper instead; the helper's name documents it.

### Sync Rules
- Logic change → update relevant symbol doc only if invariant/contract changed. Don't touch doc for cosmetic refactors.
- Adding/removing modules or crates or changing key flows MUST update SYSTEM_MAP.md in same session.
- FORBIDDEN: changing public contract (function signature, trait, error type) without updating its Rustdoc.

## Performance & Safety Standards (MANDATORY)

### Minimum Resource Cost
Design queries, computation, and data access with minimum I/O, minimum blocking of async executor threads, memory efficiency (avoid unnecessary clones, prefer borrowing).

### Required Evaluations
- **Blocking Safety**: CPU-heavy work or blocking syscalls inside async code → move to `spawn_blocking` or a dedicated worker thread pool
- **Network Cost**: Payload size, caching, connection pooling, pagination/streaming for large responses
- **Database Selectivity**: Proper index usage for queries; avoid loading full tables into memory
- **Allocation Discipline**: Avoid unnecessary `.clone()`, prefer `&str`/`&[T]` over owned types in signatures where lifetime allows, use `Cow` when either path is plausible

### Anti-Waste
- No N+1 queries (use joins/aggregation)
- No polling when async notification (channels, pub/sub) is available
- Debounce/throttle on high-frequency inputs (file watchers, retry loops)
- Avoid unnecessary `Arc<Mutex<T>>` when a channel or single-owner pattern suffices

### Contextual Strategy
Choose concurrency model per context (async task, thread pool, actor via channel), not one pattern for all.

### Scalability
- Use streaming iterators / async streams for large datasets instead of collecting into `Vec` upfront
- Handle process lifecycle (graceful shutdown, signal handling) for state consistency

### Heavy Operation Justification
Before finalizing heavy computation, large in-memory buffers, or complex concurrent state: briefly explain efficiency rationale, latency/throughput trade-off, and contention risk avoided.

## Project-Specific

- Use `cargo` workspace commands (`cargo build --workspace`, `cargo test --workspace`) unless a specific crate is targeted
- Always git blame specific lines before attributing code to a developer
- Fix ALL instances of a bug pattern at once
- Fix deprecated APIs or clippy lint warnings when encountered on same file, even if not part of the current task
- Never include Co-Authored-By or Claude credit in commits
- **NEVER run `git push` unless the user explicitly asks**. Applies to every branch including feature branches. Commit locally; the user decides when to push. Pushing to `main` without explicit ask is a hard violation, even after a "create pr" instruction, push only the branch the PR will target, never main directly. Before any `git push`, also re-check the current branch (`git branch --show-current`); if it's `main`, stop and tell the user.