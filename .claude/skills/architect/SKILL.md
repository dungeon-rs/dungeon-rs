---
name: architect
description: Derive the project's needs, research technology against them, and design a volatility-based architecture; or change the architecture.
disable-model-invocation: true
argument-hint: "[change <component or rule>]"
---

Produces `docs/architecture/NEEDS.md` and `docs/architecture/ARCHITECTURE.md`. Research leaves no files: findings come back in agents' replies and are folded into these documents. The method is Juval Löwy's volatility-based decomposition, followed literally: see [LOWY-RULES.md](./LOWY-RULES.md). File formats: [ARCHITECTURE-FORMAT.md](./ARCHITECTURE-FORMAT.md).

**The measure of this architecture: a new capability is a spec, not a design session.** Every step below serves that.

Two modes, chosen by the arguments: **full run** (`/architect`) and **change** (`/architect change`).

## Rules for every mode

- **Technology-agnostic until the mapping step.** Needs, core use cases, volatilities, and components never name a technology.
- **The architecture is meant to be stable.** Adding a component, operation, or rule is normal. Changing or removing one is an architecture change (see Change).
- **No ADRs.** A decision that would surprise a future reader gets a single `_Why_:` line.
- **Split files** only when one outgrows a screen or two.
- **Facts are found by sub-agents; decisions are the user's.** Use the Skill tool for "grilling" whenever decisions are put to the user.

## Full run

**Preconditions:** `docs/PRODUCT.md` exists and `docs/domain/` is settled (no `docs/domain/OPEN-QUESTIONS.md`). Otherwise stop and point the user at `/domain`.

### 1. Needs

Draft needs from four sources, and name the source on each:

- domain invariants ("nothing is fixed at creation" → every property stays editable after creation)
- domain Commands ("Export Report" → render a report off-screen at an arbitrary size)
- `docs/PRODUCT.md` principles ("fast at any catalogue size")
- the functional baseline's pain points in `docs/PRODUCT.md`, if a baseline is named there (quote the pain point in the Need's source)

Then ask the user two things: **"What failed before?"** (in earlier attempts at this project, if any) and **"Which technologies are on the shortlist?"** (if none, the research proposes candidates per area). Needs that come from a previous failure are Musts with source *previous attempt*. Record the shortlist at the top of `docs/architecture/OPEN-QUESTIONS.md`.

Put the draft to the user as a grilling round: confirm, prune, and mark each need **Must** (unmet is a blocker) or **Want** (can be filled later). Write `docs/architecture/NEEDS.md`.

### 2. Research and decomposition

Research runs in the background. While it runs, draft the decomposition (decomposition steps 1–4 below) without asking anything, drafting components alongside the volatilities: mapping them onto components is what exposes a volatility list that needs refining. Keep the draft and every pending question in `docs/architecture/OPEN-QUESTIONS.md`; with the shortlist and `NEEDS.md` already written down, a lost agent can be relaunched from them. An agent interrupted mid-work is resumed with its context (send it a message) rather than relaunched. When every research agent has reported, put the draft decomposition to the user in one round, together with any questions the research raised.

**Research.** Dispatch one background sub-agent per technology area the Needs touch (for example runtime, UI, persistence, and any other area the Needs touch). Brief each with the shortlist, the whole of `NEEDS.md`, and this contract:

- For every Need and every candidate: ✅ solved, ⚠️ a gap we fill ourselves (with rough effort), or ❌ blocker.
- Tag every verdict: *built* (verified by building or running code), *read* (docs or source), or *inferred*.
- Include "build it ourselves" wherever a gap exists.
- Spike any Must that is ❌ or ⚠️ and matters to the architecture, Musts from a previous attempt first: throwaway code in a scratch directory the agent deletes when done. A spike ends in a conclusion; its code and transcript are not kept.
- Reply with the verdict table, a recommendation, and the conclusion of each spike. Write no files. The main session folds the result into `NEEDS.md` and `ARCHITECTURE.md` at once, with a `_Why_:` line for what would surprise a reader.

**Decomposition**, drafted while research runs, then confirmed with the user:

1. **Core use cases**: propose 2–6 from the domain's Commands and workflows; the user confirms. A core use case without a matching domain workflow gets one added first, via the Skill tool for "domain-modelling".
2. **Volatilities**: list what is likely to change, each tagged with its axis: *over time* (the same user's needs evolving) or *between users* (different users wanting different things at once). Surface hidden ones by restating solutions as requirements and by comparing with the baseline. Volatility is open-ended; bounded variation is not a volatility.
3. **Components**: one per volatility. Mapping volatilities onto components often refines the list (a sequence volatility per Manager, merged or dropped entries); put any change to a confirmed list back to the user, old and new side by side. Each component is typed Manager, Engine, ResourceAccess, Utility, or Client, per [LOWY-RULES.md](./LOWY-RULES.md); the composition root (Host) and the shared data contracts (Model) are structural units, not components. Contracts expose business verbs, 3–5 operations each. Apply the sizing smell test: 2–5 Managers, 2–3 Engines; more Managers means the decomposition has become functional.
4. **Command ownership**: every domain Command is handled by exactly one Manager.

### 3. Mapping onto technology

When the research lands, consolidate it into one decision table (Needs × candidates) and put the choice to the user. Record the chosen technology with `_Why_:` lines, then:

- **Project tasks**: the foundational setup provides the `just` recipes `check`, `test`, and `run` (see the `implement` skill); record here what `check` runs.
- **Framework boundary**: for every component, whether it may depend on the chosen framework or runtime. This is always explicit.
- **Rule translation**: if the framework's data model is the domain model, write a table translating each Löwy rule into the framework's terms, marking each **enforced** (by a check the project's `just check` runs) or **review** (left to the review agent). Be honest about which rules become conventions.
- **Dependencies**: tables of the project's units (the compile or deployment unit that maps to a component: package, assembly, module, ...), their types, their allowed dependencies, and restricted external dependencies. Say in the Technology section what the unit is, what form a test reference takes, and which check enforces these tables, if any.

Write `docs/architecture/ARCHITECTURE.md` from the confirmed decomposition and the decisions above, in the shape of [ARCHITECTURE-FORMAT.md](./ARCHITECTURE-FORMAT.md).

### 4. Validation

- **Call chains**: one Mermaid diagram per core use case, through the components. Look for symmetry; a chain that breaks the communication rules means the decomposition is wrong, not the rule.
- **Future-change test**: pick 3–5 plausible future features (from `PRODUCT.md`, parked spec questions, the baseline, and the volatilities). Trace each: it must fit by adding behaviour to existing components, such as a new operation on a contract or a new Engine rule, with no restructuring. If one needs a restructure, a volatility was missed: go back to decomposition now. Record each future feature and how it fits.

### 5. Close

1. Self-check: every Must is met by a component or a technology decision, or explicitly accepted by the user as a gap; every domain Command has exactly one owning Manager; every volatility is encapsulated by exactly one component; no component exists without a volatility (Utilities, the Host, and the Model excepted); every future feature has a recorded fit.
2. Run `just check` if the project has a `justfile` and code.
3. Move any spec-level question still open to `docs/specs/OPEN-QUESTIONS.md`, then delete `docs/architecture/OPEN-QUESTIONS.md`.
4. Summarize for the user: components by type, the technology decisions, the rules left to review, the future features and how they fit, and any accepted gaps.

## Change

1. Say plainly that this is an architecture change, quoting old and new side by side:
   > **Architecture change: {component or rule}.** It says: *"{old}"*. This makes it: *"{new}"*.
2. Dispatch a sub-agent to list every spec, change spec, and unit affected.
3. Say whether a volatility was missed. Frequent changes mean the decomposition is wrong; say so rather than patching.
4. Get the user's explicit confirmation, then edit `ARCHITECTURE.md` and the Dependencies tables in the same commit as any code or check configuration they govern.
