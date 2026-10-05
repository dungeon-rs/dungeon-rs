# dungeon-rs

A fast map editor for tabletop role-playing games, built in Rust on Bevy. Built with a spec-driven method: `docs/METHODOLOGY.md`.

## Where things live

- `docs/PRODUCT.md`: vision and principles every change respects.
- `docs/domain/`: the domain language and invariants. Use its terms everywhere; never invent synonyms.
- `docs/architecture/`: what the system must satisfy (`NEEDS.md`) and how it is decomposed (`ARCHITECTURE.md`).
- GitHub milestones and issues: the ordered plan and the backlog, the single source of truth (read them with `gh`; format in `.claude/skills/roadmap/PLAN-FORMAT.md`). `docs/changes/`: specs of work in progress. `docs/specs/`: what the system does now.
- `docs/guidelines/`: how to build recurring building blocks.
- `.claude/skills/`: the workflow. Entry points: `/domain`, `/architect`, `/roadmap`, `/spec`, `/implement`, `/guideline`.

## When X, read Y

- Writing or changing code: `docs/guidelines/INDEX.md`, then `docs/architecture/ARCHITECTURE.md` (the component you are in, its contract, and the Dependencies tables).
- Touching behaviour in a capability: its pinned spec in `docs/specs/<capability>/SPEC.md`, and the change spec in `docs/changes/` if one is in progress.
- A term is unclear or seems missing: `docs/domain/`. Missing concepts go through `domain-modelling`, never straight into code or specs.
- A choice seems to need a new component, contract, or dependency direction: stop; that is `/architect change`.

## Conventions

- **Current state only.** Docs, specs, and guidelines describe what is true now. History lives in git: no changelogs, "previously", or superseded copies.
- **No IDs and no pointers.** Specs, stories, and rules have names, never numbers. Branches are `<type>/<what-it-does>`; commits and pull-request titles are Conventional Commits that fully state what they are about, never referring to IDs or transient files. Issue numbers appear only on GitHub: a pull-request body closes its issue, and nothing in the repository names one.
- **Nothing is pushed without explicit authorization.** Pull requests open as drafts; the author merges.
- **Links go from spec to test only.** Code never refers to specs.
- **Facts are found, decisions are asked.** Look things up (or send a sub-agent) instead of asking. Put decisions to the author in one consolidated round, and only after all background work has finished; never add questions to an open round.
- **Nothing lands without passing the gates**: every configured check with fresh output, the two-axis review, and the author's manual check.
- Oxford English. The default branch is `master`.
- Research and spikes leave no files: findings are folded into tracked documents at once. The gates run through `just` (`just check`, `just test`, `just run`); the `justfile` holds the project's commands.
