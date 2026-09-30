---
name: roadmap
description: Plan or replan the order of changes in docs/ROADMAP.md as vertical slices grouped into milestones.
disable-model-invocation: true
---

`docs/ROADMAP.md` is the ordered plan of changes: milestones stated as outcomes, each with the changes that reach it. Format: [ROADMAP-FORMAT.md](./ROADMAP-FORMAT.md). Change specs are not written here; `/spec` writes one when building of a change starts.

Run it at the start of each milestone, and whenever a landed change teaches something that affects the plan.

## Rules

- **Vertical slices.** Every change delivers behaviour a user of the product can try, except the foundational setup. Component or layer order applies only inside a change.
- **Foundational setup, then a walking skeleton.** When nothing is built, the first change is the foundational setup: the repository, `CLAUDE.md` (spelling convention, default branch), the `justfile` with `check`, `test`, and `run` (defined in the `implement` skill), and the project's static analysis and CI. The next is the thinnest end-to-end slice through every component, e.g. create one item, change it, persist it, read it back.
- **Order: dependencies, then risk, then value.** A change never precedes one it builds on; among the rest, the riskiest goes first; ties go to what makes the product more useful soonest. Wants come after Musts unless the user promotes one.
- **Risky unknowns are question lines.** When a change depends on something unproven, put a line whose outcome is the question ("Answer: can the report be rendered fast enough at 10,000 rows?") just before it. It ends in a conclusion folded into the domain, the change spec, or the architecture (with a `_Why_:` line), and the line is then deleted. Any spike code is thrown away.
- **One PR a human can review.** Every change fits a single pull request of reasonable size. Split anything larger now, not mid-implementation. The foundational setup is the accepted exception.
- **Plan near, sketch far.** The current and the next milestone list their changes; later milestones are a heading and an outcome only.
- **Current plan only.** Landed changes and finished milestones are removed; history lives in version control.

## Plan

0. **Preconditions.** `docs/architecture/NEEDS.md` and `docs/architecture/ARCHITECTURE.md` exist; otherwise stop and point the user at `/architect`.
1. **Read** the domain Commands (`docs/domain/`), `docs/architecture/NEEDS.md`, `docs/architecture/ARCHITECTURE.md` (components, future changes, accepted gaps), `docs/specs/OPEN-QUESTIONS.md`, existing pinned and change specs, and the current `docs/ROADMAP.md`.
2. **Find the gaps**: list every domain Command and compare it with the roadmap lines and the specs. A Command that no spec owns and no roadmap line lists is *unplanned*; each is either placed on the roadmap in this run or listed on a sketched later milestone.
3. **Draft** the milestones and changes following the rules above. For each change: a title, the outcome in one line, and the domain Commands it covers.
4. **Size check** each change against a single reviewable PR; split where needed.
5. **Put the draft to the user** as one round (call the Skill tool for "grilling"): milestone outcomes, the order and why, the question lines, and any split. Write only what the user confirms.
6. **Write** `docs/ROADMAP.md` and repeat the comparison from step 2: no Command should be unplanned.
