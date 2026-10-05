---
name: implement
description: Build the next change from the plan on GitHub, from issue to merged pull request, with automated, independent review, and author gates.
disable-model-invocation: true
argument-hint: "[change title or issue, default: the next open change]"
---

Build one change end to end in this session. The main session writes the code; sub-agents only review and look things up. The author is in the loop at the spec, at the offer of review before merging, and whenever a stop condition fires.

## The project's tasks

Skills know nothing about the project's stack. They run its tasks through `just`, which the project provides:

- `just check`: the whole automated gate: build, tests, lints, static analysis, and any architecture or dependency enforcement the project has. Exits non-zero on failure.
- `just test`: tests only, for fast feedback while building.
- `just run`: launches the app or service for the manual gate.

If a needed recipe is missing, stop and ask; don't guess a command. The foundational setup is the exception: it provides the recipes, so write them first.

## 1. Pick and classify

Take the change named in the arguments, or the next open Feature issue: the earliest open milestone's first issue, by dependencies and then issue number, that no open issue blocks (`gh issue list --milestone <milestone> --state open`). Say out loud which issue and which path it is:

- **Spike**: answers a question. No spec. Work in a scratch directory. It must end in a conclusion. Then delete the spike code, fold the conclusion into the domain, the architecture (with a `_Why_:` line), or the plan's issues. Nothing about the spike is kept. Leave the edits uncommitted for the user, propose closing the answered question issue with the conclusion as its closing comment, and stop.
- **Bounded**: the normal path below.
- **Architectural**: needs a component, contract, or rule changed. Stop and ask the user to run `/architect change` first.

When in doubt, take the heavier path. Never downgrade mid-task.

For a bounded change, assign the issue to the author's account (`gh issue edit <n> --add-assignee @me`): that is what says it is being worked on.

## 2. Branch

Create a branch named `<type>/<issue-number>` with its Conventional Commit type, e.g. `feat/12`; the issue says what it does. Work on it in the checkout, or in a worktree (`git worktree add`) when sub-agents or other work run alongside. Base: the repository's default branch. Preserve any uncommitted work first; never discard it.

## 3. Spec

Read `.claude/skills/spec/SKILL.md` and follow its Write mode for the change. The test seams are agreed with the user there. Commit the change spec (call the Skill tool for "commit"). Then push the branch and open a **draft** pull request with `gh pr create --draft`:

- Title: a Conventional Commit title, e.g. `feat: place items from a library`.
- Body: [PR-FORMAT.md](./PR-FORMAT.md), filled from what is known so far, closing the change's issue (`Closes #<n>`).

The pull request is the persistent scratchpad. A session that resumes the change reads it first.

## 4. Build

- **Tests first, only at the agreed seams**: for each Rule, a failing test, then the code that makes it pass (`just test` for the loop). No tests of internals; no tests that restate the implementation. Refactor after green.
- Read `docs/guidelines/INDEX.md` first, if it exists, and follow every guideline that applies. Follow `docs/architecture/ARCHITECTURE.md`.
- Commit in small atomic steps with the "commit" skill, and push them. Run `just commits` before every push: it is cheap, and it is the one check the gate runs on history that cannot be fixed once pushed.
- Comment on the pull request at each point a reader would want to know about: the seams agreed, a decision taken and why, the gate green, a finding that changes the plan. Long work is resumable from those comments.

## 5. Automated gate

Run `just check`. Nothing counts as passing without fresh output from this run. Fix and rerun until it passes.

## 6. Land

Read `.claude/skills/spec/SKILL.md` and follow its Land mode: the pinned spec is updated, the change spec deleted, and the landing committed and pushed. Rerun `just check` afterwards.

## 7. Ready

Start the product with `just run` to be sure it launches (run it in the background if it doesn't exit; for a library exercise it through a scratch caller or its examples). Comment on the pull request with the checklist for the author's manual check, derived from the change spec's user stories: what to do by hand (clicks, requests, commands), and what they should see. Then mark it ready (`gh pr ready`).

## 8. Independent review

Call the Skill tool for "review" on the pull request. Its sub-agents post their reviews to the pull request; read the findings there, not from the sub-agents' replies. When a review names a building block that needs a guideline, offer `/guideline` to the user before merging.

## 9. Fix and re-request

Answer each finding on the pull request: fix it in a commit and say which, or dispute it with the reason. Push, rerun `just check`, and re-request the review (the review skill's Re-review). Repeat until a pass leaves no finding open. Anything that needs a decision goes to the user in one consolidated round.

## 10. Offer and merge

Ask the user whether they want to review the pull request or do the manual check. If they do, wait; anything they report goes back to step 4. If they decline, merge with a rebase merge (`gh pr merge --rebase --delete-branch`), so the atomic commits stay on the default branch. Then confirm the issue closed; if the pull request did not close it, close it with a comment saying what landed. Remove the worktree if one was used, bring the default branch up to date, and move on to the next issue.

## Stop conditions

Stop and ask, never work around:

- a Need the architecture doesn't meet
- a concept the domain doesn't define
- an agreed test seam that can't be reached
- a spec that turns out to be wrong or incomplete: show the proposed edit to the change spec side by side, and once the user confirms it, let them choose between **reworking** the current code and **starting fresh** on a new branch from the default branch (named `<type>/<issue-number>-2`, and so on; the old branch and its pull request stay for reference and nothing is discarded)
