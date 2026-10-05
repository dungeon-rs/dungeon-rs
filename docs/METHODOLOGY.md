# Methodology

Decisions for the spec-driven skill set. Present tense: this describes the method as it stands.

## Flow

1. **Domain**: terms, invariants, commands, events, workflows. Technology-agnostic.
2. **Needs**: technology-agnostic requirements derived from the domain, `PRODUCT.md`, and the functional baseline.
3. **Tech research**: candidate technologies scored against the Needs.
4. **Architecture**: volatility decomposition from the domain's workflows, then mapped onto the chosen technology.
5. **Roadmap, then change specs**: the roadmap (milestones and issues on GitHub) is complete because the architecture exists; a change spec is written when building of its change starts.
6. **Implementation**: each change lands into its pinned spec on the same branch, before its pull request.

## Artefacts (in the project repo)

```
<project>/
├── CLAUDE.md                        ← tiny: what the project is, pointers, "when X, read Y", standing conventions
├── .claude/skills/                  ← the skill set, local to the project
├── justfile                         ← the project's tasks: check, test, run
├── docs/
│   ├── PRODUCT.md                   ← vision + principles every spec respects
│   ├── domain/                      ← DOMAIN.md + <context>.md; tech-agnostic
│   ├── architecture/                ← volatility decomposition, mapping of Commands to Managers
│   │   ├── NEEDS.md                 ← what the architecture must satisfy, each need with its source
│   │   └── ARCHITECTURE.md          ← components, technology, enforcement, dependencies
│   ├── specs/<capability>/SPEC.md   ← pinned: what the system does now
│   ├── changes/<slug>.md            ← not-yet-implemented behaviour; deleted in the landing commit
│   └── guidelines/                  ← INDEX.md + <building-block>.md
└── <source>/…
```

The plan and the backlog are not files. They live on GitHub (see **Issues are the plan** below).
## Rules

- **Current state only.** Domain, specs, and guidelines state what is true now. History lives in version control: no deltas, changelogs, or superseded copies.
- **Same-commit updates.** A change to behaviour or to a pattern updates its spec or guideline on the same branch, before the pull request.
- **Specs by capability.** A capability is a coherent set of things a user can do; every domain Command belongs to exactly one, named in the spec's header. The spec-axis review reports unclaimed and doubly claimed Commands.
- **Issues are the plan.** The roadmap and the backlog are GitHub milestones and issues, the single source of truth; the repository holds no list of planned work. A milestone is an outcome a user could state. A change is a Feature issue in its milestone; a question to answer or a piece of work that is not a change is a Task. Labels: `area:<capability>`, `spec:none|change|new` (how the work relates to the pinned specs), `needs-decision` (waiting on the author), `blocked`, and `polish`, `test-gap`, `tech-debt`. Build order is milestone order, then dependencies (blocked-by), then issue number. Landed work is closed, not edited out; history lives in GitHub and git. Format: the roadmap skill's `PLAN-FORMAT.md`.
- **The roadmap is vertical slices.** A walking skeleton first; then changes ordered by dependencies, risk, then value, grouped into milestones a user could state. Each change fits one PR a human can review (the initial setup excepted). Risky unknowns are question issues answered by research or spikes, each ending in a conclusion folded into the docs and the issue closed. Current and next milestone in detail, later ones sketched.
- **Change specs are written when building starts**, not ahead. They hold everything not yet implemented, on any branch, until they land. Pinned specs hold only what is true now.
- **No IDs.** Specs, stories, and rules have names, never numbers or codes.
- **Links go from spec to test only.** A pinned spec lists the tests covering each Rule; code never refers to specs.
- **Specs speak the domain's language.** A new concept goes into the domain first.
- **No stored maturity labels.** Rigor comes from observable facts and in-the-moment judgement:
  - Is the thing being touched covered by a pinned spec? Then the spec is updated first and checked against the code before landing.
  - Is the task a spike, bounded, or architectural? Judged when the task starts, never stored. When in doubt, the heavier path; never downgrade mid-task. Spikes never land, and nothing of them is kept.
- **Nothing lands unless its pinned spec agrees with the code.**
- **Rationale is a single `_Why_:` line**, only where a rule would otherwise surprise a reader. No ADRs; anything that needs more belongs in the architecture doc.
- **Guidelines are created lazily**: the second time a building block is written, or the first time when the architecture guarantees it repeats. They are extracted from real code, indexed in `INDEX.md` with a "use when" line, and say *how* (never what or why). The standards review flags unguided repeats and examples that have drifted from their exemplar.
- **Checks before prose.** A rule a tool can enforce becomes a lint or a step of `just check`, not a written rule.
- **Each phase hands leftovers forward in one shape.** A phase's working questions live in its own `OPEN-QUESTIONS.md`; anything that belongs to a later phase is parked as an issue labelled `needs-decision` (domain → spec-level questions).
- **Structure before behaviour.** The domain settles its things (terms, invariants) before its verbs (commands, events, workflows).
- **Research leaves no files.** Agents report in their replies; the main session folds every finding into the domain, architecture, specs, or guidelines at once, with a `_Why_:` line where a decision would surprise. Spike code is thrown away. Not sharing in-flight research between machines is an accepted trade-off.
- **Hard questions become research.** When the user can't answer from knowledge, a sub-agent researches options against a scenario suite that includes two parties on different setups where the product is shared or concurrent.
- **The architecture follows Löwy literally** (Managers, Engines, ResourceAccess, Utilities, closed layering), translated into the framework's terms when the framework's data model is the domain model. Rules no check enforces are marked "review" and left to the review agent.
- **The framework boundary is always an explicit decision**, per component.
- **An architecture is judged by the future-change test**: 3–5 plausible features must fit without restructuring. A new capability is a spec, not a design session.
- **The architecture is as stable as the domain.** Changing or removing a component, contract, or rule is a confirmed architecture change; frequent ones mean a missed volatility.
- **The main session writes the code**; sub-agents review and look things up. A change passes an automated gate (fresh output of every configured check), a two-axis review, and a manual check by the author before its draft pull request. The author merges.
- **No pointers in git.** Branches are `<type>/<what-it-does>`; commits and pull-request titles are Conventional Commits that fully state what they are about, never referring to IDs or transient files. Issue numbers appear only on GitHub, in a pull-request body that closes its issue. Nothing is pushed without the author's explicit authorization.
- **The domain is meant to be stable.** Additions are normal; changing or removing a definition or invariant is an explicit, confirmed domain change.

## Skills

| Skill | Invocation |
|---|---|
| `domain` | user |
| `domain-modelling` | model |
| `grilling` | model |
| `architect` | user |
| `spec` | user |
| `roadmap` | user |
| `guideline` | user |
| `implement` | user |
| `review` (spec axis + standards axis) | model |
| `commit` | model |

The skills are project-agnostic: everything specific to a project's choices lives in `docs/` and the `justfile`, and they run its tasks through `just`. Built on [mattpocock/skills](https://github.com/mattpocock/skills) (MIT), adapted.
