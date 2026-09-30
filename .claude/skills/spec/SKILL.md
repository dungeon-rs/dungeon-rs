---
name: spec
description: Write a change spec for a capability, or land a finished change into its pinned spec.
disable-model-invocation: true
argument-hint: "<capability or change> | land <change>"
---

Specs describe behaviour in domain language. There are two kinds, and they never mix:

- **Change spec** (`docs/changes/<slug>.md`): behaviour that is **not implemented yet**. Lives until it lands, then is deleted in the landing commit. Template: [CHANGE-TEMPLATE.md](./CHANGE-TEMPLATE.md).
- **Pinned spec** (`docs/specs/<capability>/SPEC.md`): what the system does **now**, and nothing else. Edited in place; history lives in version control. Template: [PINNED-TEMPLATE.md](./PINNED-TEMPLATE.md).

A **capability** is a coherent set of things a user can do, built around one or two domain concepts. Every domain Command belongs to exactly one capability; a spec's header names the Commands it owns.

Two modes, chosen by the arguments: **write** (`/spec <capability or change>`) and **land** (`/spec land <change>`).

## Rules for every mode

- **Domain terms only.** Use the language in `docs/domain/`. If a spec needs a concept the domain doesn't define, stop and call the Skill tool for "domain-modelling" to add it first.
- **No IDs.** No numbers or codes for specs, stories, or rules. Rules get short bold names, like domain invariants.
- **Links go from spec to test, never back.** Code, comments, attributes, and commit messages never refer to specs.
- **Pinned specs hold no intent.** Nothing planned, pending, or "to do" in `docs/specs/`. That belongs in a change spec.
- **Never write around the architecture.** If a story can't be met within `docs/architecture/`, stop and raise it with the user as an architecture question, naming the need it exposes.
- **No file paths or code in Implementation Decisions.** Name modules and interfaces; test references belong only in Test seams (Testing, in a change spec).
- **Tests are referenced in the project's form**, as documented in the Technology section of `docs/architecture/ARCHITECTURE.md`.

## Write

1. **Preconditions.** `docs/architecture/` must exist; if it doesn't, stop and tell the user to run `/architect` first. Read `docs/PRODUCT.md`, `docs/domain/`, `docs/architecture/` (including `NEEDS.md`), `docs/specs/OPEN-QUESTIONS.md`, `docs/ROADMAP.md`, and the pinned specs of the capabilities involved.
2. **Scope.** Name the capability or capabilities the change touches and the domain Commands each owns. A Command already owned by another capability's spec stays there unless the user decides to move it.
3. **Interview or synthesize.** If the conversation already covered this behaviour, synthesize. Otherwise call the Skill tool for "grilling" and seed its frontier with:
   - the Commands the capability owns, and what each must do
   - items in `docs/specs/OPEN-QUESTIONS.md` that belong to this capability
   - how the functional baseline (named in `docs/PRODUCT.md`, if any) handles it, and the pain points recorded as sources in `docs/architecture/NEEDS.md`
   - existing Rules of the pinned spec that this change would modify
4. **Draft** the change spec from [CHANGE-TEMPLATE.md](./CHANGE-TEMPLATE.md):
   - User Stories: extensive, one behaviour each, in domain terms.
   - Rules: every behaviour a test could check, each with a short bold name. Name the domain invariant (its bold name, in plain text) when a Rule follows from it.
   - Changes to existing behaviour: every existing pinned Rule this change modifies or removes, by name.
   - Out of Scope: what this change deliberately leaves for later.
5. **Architecture check.** Walk each story against `docs/architecture/`. Anything it can't meet goes to the user as an architecture question before the spec is written.
6. **Agree the test seams** with the user: the highest seam that exercises the behaviour, and as few seams as possible.
7. **Write** `docs/changes/<slug>.md` and link its line in `docs/ROADMAP.md` to it (`- **[<title>](changes/<slug>.md)**: …`). If the change isn't on the roadmap, stop and ask whether to run `/roadmap` first. Remove the items the spec settled from `docs/specs/OPEN-QUESTIONS.md`.
8. **Check coverage** yourself, for the Commands this change touches: each is defined by the domain, each is owned by exactly one capability (a Command the change moves is listed under "Changes to existing behaviour"), and each is on the roadmap. Fix or raise anything that isn't.

## Land

Carried out by `implement` (it reads this file) after the automated, review, and manual gates have passed; the result is committed on the change's branch before the pull request.

1. Read the change spec and every pinned spec it names. A capability without a pinned spec gets one now: that is the moment it becomes pinned.
2. For each capability, rewrite the pinned spec from [PINNED-TEMPLATE.md](./PINNED-TEMPLATE.md) so it states what is true now:
   - stories in present tense ("As a {role}, I can…")
   - new Rules added; Rules listed under "Changes to existing behaviour" modified or removed
   - Implementation Decisions describing the system as built, not the plan
   - Test seams: for each Rule, the tests that cover it, in the project's test-reference form
   - Not supported: only lasting non-goals of the capability; "not in this change" items are dropped
3. Delete the change spec and its line in `docs/ROADMAP.md`, and any milestone left empty.
4. Commit the landing (a message that fully describes the change, never pointing at the change spec) with the Skill tool for "commit".
5. Call the Skill tool for "review", spec axis only, against the merged pinned spec(s). Fix or raise every finding before handing back.
