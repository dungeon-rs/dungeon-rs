---
name: implement
description: Build the next change from the roadmap, from change spec to a draft pull request, with automated, review, and manual gates.
disable-model-invocation: true
argument-hint: "[change title, default: top of the roadmap]"
---

Build one change end to end in this session. The main session writes the code; sub-agents only review and look things up. You are in the loop at the spec, at the manual check before the PR, at the push authorization, and whenever a stop condition fires.

## The project's tasks

Skills know nothing about the project's stack. They run its tasks through `just`, which the project provides:

- `just check`: the whole automated gate: build, tests, lints, static analysis, and any architecture or dependency enforcement the project has. Exits non-zero on failure.
- `just test`: tests only, for fast feedback while building.
- `just run`: launches the app or service for the manual gate.

If a needed recipe is missing, stop and ask; don't guess a command. The foundational setup is the exception: it provides the recipes, so write them first.

## 1. Pick and classify

Take the change named in the arguments, or the top line of `docs/ROADMAP.md`. Say out loud which path it is:

- **Spike**: answers a question. No spec. Work in a scratch directory. It must end in a conclusion. Then delete the spike code, fold the conclusion into the domain, the architecture (with a `_Why_:` line), or the roadmap, and delete the answered question line from `docs/ROADMAP.md`. Nothing about the spike is kept. Leave the edits uncommitted for the user and stop.
- **Bounded**: the normal path below.
- **Architectural**: needs a component, contract, or rule changed. Stop and ask the user to run `/architect change` first.

When in doubt, take the heavier path. Never downgrade mid-task.

## 2. Branch

Create a branch named after the work with its Conventional Commit type: `<type>/<what-it-does>`, e.g. `feat/place-items-from-a-library`. No IDs, and no reference to the change spec's file name. Base: the repository's default branch. Preserve any uncommitted work first; never discard it.

## 3. Spec

Read `.claude/skills/spec/SKILL.md` and follow its Write mode for the change. The test seams are agreed with the user there. Commit the change spec (call the Skill tool for "commit").

## 4. Build

- **Tests first, only at the agreed seams**: for each Rule, a failing test, then the code that makes it pass (`just test` for the loop). No tests of internals; no tests that restate the implementation. Refactor after green.
- Read `docs/guidelines/INDEX.md` first, if it exists, and follow every guideline that applies. Follow `docs/architecture/ARCHITECTURE.md`.
- Commit in small atomic steps with the "commit" skill.

## 5. Automated gate

Run `just check`. Nothing counts as passing without fresh output from this run. Fix and rerun until it passes.

## 6. Review

Call the Skill tool for "review". Fix each finding or raise it with the user; none is dismissed silently. When the review names a building block that needs a guideline, offer `/guideline` to the user before landing. Rerun the automated gate after fixes.

## 7. Manual gate

Start the product with `just run` (a UI, a service, or a command line; run it in the background if it doesn't exit, and for a library exercise it through a scratch caller or its examples). Give the user a short checklist derived from the change spec's user stories: what to do by hand (clicks, requests, commands), and what they should see. Wait. Anything the user reports goes back to step 4.

## 8. Land

Read `.claude/skills/spec/SKILL.md` and follow its Land mode: the pinned spec is updated, the change spec deleted, the roadmap line removed, and the landing committed and reviewed. Rerun `just check` afterwards.

## 9. Pull request

Ask the user to authorize the push; stop until they do. Then push and open a **draft** pull request with `gh pr create --draft`:

- Title: a Conventional Commit title, e.g. `feat: place items from a library`.
- Body: [PR-FORMAT.md](./PR-FORMAT.md), filled from the actual diff and history.

The user merges; never merge yourself.

## Stop conditions

Stop and ask, never work around:

- a Need the architecture doesn't meet
- a concept the domain doesn't define
- an agreed test seam that can't be reached
- a spec that turns out to be wrong or incomplete: show the proposed edit to the change spec side by side, and once the user confirms it, let them choose between **reworking** the current code and **starting fresh** on a new branch from the default branch (the old branch stays for reference; nothing is discarded)
