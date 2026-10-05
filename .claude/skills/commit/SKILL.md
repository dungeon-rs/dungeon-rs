---
name: commit
description: Organize pending changes into atomic Conventional Commits with messages that stand on their own. Use whenever committing work, including from other skills.
---

Commit pending work as atomic Conventional Commits. Preserve the user's work above all.

## Safety

- Inspect repository state before changing anything.
- Never bypass hooks with `--no-verify`.
- Never use destructive commands: `git reset --hard`, `git clean`, `git checkout --`, `git restore` on working-tree content, `git branch -D`.
- Never force-push. Never push unless the user explicitly authorized it; committing never includes pushing.
- Never amend, rebase, squash, or cherry-pick unless the user asked for it.
- Stop and explain on conflicts, unmerged paths, a detached HEAD, an in-progress merge/rebase/cherry-pick, or an unexpected divergence.
- Never commit secrets, credentials, or local configuration; skip them and warn the user.

## Messages

- Conventional Commits: `type: subject`. No scope unless the work has a distinctly defined one.
- Types: `feat`, `fix`, `docs`, `refactor`, `chore`, `test`, `style`, `perf`, `build`, `ci`, `revert`.
- Subject: imperative, lowercase first letter, no trailing period, and within the project's own commit lint (`committed.toml`, run by `just commits`; its default of 50 characters applies when it sets none), falling back to 72 characters only when the project has no lint. In the project's spelling convention (see `CLAUDE.md`).
- **No pointers.** No IDs, ticket numbers, or references to transient files (such as change specs). A message fully states what the commit is about on its own; a reader with only `git log` understands it.
- Every non-trivial commit has a body, separated by a blank line and wrapped at 72 characters, that explains why the change is needed, the constraints or trade-offs, and notable consequences. It does not restate the diff. A subject-only commit is for a small, self-explanatory, single-purpose change such as a typo fix.
- Breaking changes: `!` after the type and/or a `BREAKING CHANGE:` footer.
- Match the repository's living convention: check `git log --format='%s%n%b%n---' -20` first.

## Workflow

1. Inspect `git status --porcelain=v1`, `git diff`, `git diff --staged`, the current branch, and recent history.
2. Nothing to commit: say so and stop. Conflicts or an in-progress operation: stop and explain.
3. Group the changes into coherent atomic commits. Keep what one logical change needs together; separate unrelated features, fixes, documentation, tests, and configuration. Don't split artificially.
4. Present the plan (each `type: subject` and its files) as a heads-up, not an approval gate, unless the user asked to review it first.
5. For each group: stage only that group (`git add -p` when a file mixes changes), verify with `git diff --staged --stat`, write the message to a file and lint it with the project's linter (`committed --commit-file <file>`) before committing, commit, and let hooks run. A message the linter refuses is fixed before the commit exists, never after it is pushed.
6. Verify with `git log --oneline` and `git status`. Report the new commits and anything intentionally left uncommitted.
