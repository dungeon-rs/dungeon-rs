---
name: review
description: Review the current change on two independent axes, spec conformance and standards, with fresh sub-agents. Use before landing a change, or on request.
---

Two fresh sub-agents review the change in parallel. Each gets only what its axis needs; neither sees the other's report. Hand both reports back as they are: don't merge, rerank, or drop findings.

The caller may ask for one axis only (e.g. `spec` after landing).

## Inputs

- The diff against the base branch (`git diff <default-branch>...HEAD`, where the default branch is the repository's own, or the base the user names), and the branch's commit messages (`git log <default-branch>..HEAD`).
- The change spec in `docs/changes/` before landing; after landing, the pinned spec(s) it merged into (the change spec is then gone, and its deletion is in the diff).

## Spec review

Brief a fresh sub-agent with the change spec, the pinned spec(s) if landed, `docs/domain/`, the Test reference bullet of the Technology section in `docs/architecture/ARCHITECTURE.md`, the diff, the commit messages, and the tests it touches. It reports:

- **Missing**: a story or Rule the code doesn't deliver.
- **Extra**: behaviour or scope the spec doesn't ask for.
- **Wrong**: code that contradicts a Rule or a domain invariant (`docs/domain/`).
- **Tests**: any test listed for a Rule that doesn't actually exercise it; any Rule without a test at an agreed seam.
- **After landing**: anything the pinned spec states that the code doesn't do, and anything the code does that no Rule states.

It also runs this coverage checklist over the whole repository, because no tool does. Read the Commands in `docs/domain/`, `docs/specs/`, `docs/changes/`, and the open Feature issues (`gh issue list --state open --json number,title,body,milestone`):

- every domain Command is owned by exactly one capability (named in a spec header), or is on an open issue's `Commands` line; report unowned, unlisted Commands as unplanned, and Commands claimed by two capabilities as errors (a Command listed under "Changes to existing behaviour" of a change spec that moves it is not a double claim)
- every spec header and every issue's `Commands` line names only Commands the domain defines
- every Rule in every pinned spec lists at least one test, and every listed test exists
- no source file, comment, attribute, or commit message refers to a spec (search source and commit messages for `docs/specs`, `docs/changes`, `SPEC.md`, change slugs, and quoted Rule names)
- every change-spec link in an issue resolves to an existing change spec, every change spec is linked from an open issue, and no pinned spec holds anything planned, pending, or "to do"

## Standards review

Brief a fresh sub-agent with:

- the diff and the commit messages
- the change spec or pinned spec(s), `docs/architecture/NEEDS.md`, and the Components section of `docs/architecture/ARCHITECTURE.md`
- `docs/guidelines/` (index first)
- the architecture: the Communication rules, Technology (including its Enforcement bullet), Dependencies, and Restricted external dependencies sections of `docs/architecture/ARCHITECTURE.md`, and [LOWY-RULES.md](../architect/LOWY-RULES.md)
- **quoted verbatim**, every row of the Rule translation table (if the project has one) not enforced by a check in the project's `just check`. What the Enforcement bullet doesn't list as enforced is guarded by this review alone, whether or not a translation table exists.

It reports:

- violations of those architecture rules, with the rule quoted
- departures from guidelines, naming the guideline
- code smells worth fixing (duplication, long functions, feature envy, primitive obsession, speculative generality, dead code, misleading names)
- anything that works around a spec, the architecture, or a Need instead of raising it
- a building block written for the second time without a guideline (or the first time, when the architecture guarantees it repeats): name it, so the caller can offer `/guideline`
- a rule a tool could enforce but which is only followed by convention: propose the lint or check

It also runs this consistency checklist:

- every guideline file is listed in `docs/guidelines/INDEX.md`, and every index entry has a file
- every guideline's Exemplar path exists, and its example still matches the exemplar's code
- the Dependencies and Restricted external dependencies tables match the project's actual component dependencies, wherever `just check` doesn't already enforce them

## Each finding

File and line, the rule or spec text it breaks, and a one-line suggested fix. Uncertain findings are marked as such, never dropped. The caller fixes each finding or raises it with the user; none is dismissed silently.
