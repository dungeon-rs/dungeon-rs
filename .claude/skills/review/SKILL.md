---
name: review
description: Review a change on two independent axes, spec conformance and standards, with fresh sub-agents that post their reviews to the pull request. Use on a ready pull request, or on request.
---

Two fresh sub-agents review the change in parallel. Each gets only what its axis needs; neither sees the other's report. Don't merge, rerank, or drop findings.

The caller may ask for one axis only.

## Where the review goes

A change under review is a pull request, and the reviewers behave as independent human reviewers would: each sub-agent posts its report to the pull request itself (`gh pr review <n> --comment --body-file <file>`), headed with its axis, the commit it reviewed, and that an independent sub-agent wrote it. One account holds every comment, and GitHub does not let it approve or request changes on its own pull request, so a review is always a comment review and its verdict is its first line: `no open findings` or `n open findings`. The main context gets only that verdict and the link, and reads the findings from the pull request (`gh pr view <n> --comments`) like any other feedback. Without a pull request (a spike, a docs-only change the author commits to the default branch), the reports come back to the caller as they are.

## Inputs

- The pull request's diff and commit messages (`gh pr diff <n>`, `gh pr view <n> --json commits`), or without one the diff against the base branch (`git diff <default-branch>...HEAD`) and `git log <default-branch>..HEAD`.
- The change's issue, for its outcome and its `Commands` line.
- The change spec in `docs/changes/`; once the landing has deleted it, read it as it stood in the commit before (`git show <commit>^:docs/changes/<slug>.md`), together with the pinned spec(s) it merged into.

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

File and line, the rule or spec text it breaks, and a one-line suggested fix. Uncertain findings are marked as such, never dropped. The main context answers each on the pull request (fixed in a named commit, or disputed with the reason) and raises what it cannot settle with the author; none is dismissed silently.

## Re-review

After fixes the main context re-requests the review. A fresh sub-agent per axis reads the earlier review comments and the commits since, reports each earlier finding as resolved or still open with its reason, and adds only new findings in the changed code. A pass with no open finding ends the loop.
