# Workspace foundation

**Capabilities**:
- Workspace: none (no Commands; it owns the rules every crate and every change is checked against)

## Problem Statement

A contributor has a toolchain and lints, but the workspace holds only the Host. The decomposition in `docs/architecture/` is prose that nothing enforces, there is no single command that runs the whole gate, and there is no CI, so every later change would be built on rules that only a reviewer's attention guards.

## Solution

Every component exists as an empty crate, `just check` is the one command that runs the whole gate (exactly what CI runs), the feature check and an architecture check enforce the workspace rules, and CI and Dependabot keep the gate and the dependencies current.

## User Stories

1. As a contributor, I want every component in the architecture to exist as an empty crate, so that later changes fill in crates instead of creating them.
2. As a contributor, I want every crate to declare `default` and `dev` features, so that debug tooling is switched on the same way everywhere.
3. As a contributor, I want `dev` to propagate to every workspace dependency, so that enabling it on the Host enables it everywhere.
4. As a contributor, I want each crate's README to document its features, so that I can find what a feature does without reading the manifest.
5. As a contributor, I want the build to fail when a crate depends on a workspace crate its row in the Dependencies table does not allow, so that the layering cannot erode unnoticed.
6. As a contributor, I want the build to fail when a workspace crate has no row in the Dependencies table, so that a new component is a deliberate architecture change.
7. As a contributor, I want the build to fail when a restricted external crate is used outside the components allowed it, so that the UI, the render stack, and the asset system stay confined.
8. As a contributor, I want Engines never to depend on Engines, ResourceAccess never on ResourceAccess, and Managers never on Managers, so that closed layering holds.
9. As a contributor, I want one `just check` that runs the whole gate, so that I never guess which checks a change must pass.
10. As a contributor, I want CI to run the same gate on Windows, macOS, and Linux for every pull request, so that platform breakage is found before merging.
11. As a contributor, I want the supply-chain check to also run weekly, so that a newly published advisory is noticed without a pull request.
12. As a contributor, I want Dependabot to propose updates for the workspace, the check tooling, and the CI workflows, so that dependencies do not rot.
13. As a contributor, I want a failing check to name the crate and the rule it broke, so that I can fix it without reading the checker.

## Rules

**Every component is a crate**: the workspace contains exactly one `drs-<component>` crate for each row of the Dependencies table, and none besides.

**Required features**: every workspace crate declares `default` and `dev`.

**Documented features**: every feature a crate declares other than `default` and `dev` appears in its README as a code-formatted name, and every crate has a README.

**Dev propagates**: a crate's `dev` feature enables `dev` on every workspace crate it depends on.

**Allowed dependencies**: a workspace crate depends only on the workspace crates listed in its row of the Dependencies table.

**Known crates only**: a workspace crate with no row in the Dependencies table fails the architecture check.

**Restricted externals**: an external crate named in the Restricted external dependencies table is a dependency only of the crates or component types listed for it.

**Well-formed tables**: a Dependencies or Restricted external dependencies row with the wrong number of cells, an unknown component type, a duplicate component, or a name that is neither a component nor a component type fails the architecture check instead of weakening it.

**Same-kind isolation**: no Engine depends on an Engine, no ResourceAccess on a ResourceAccess, no Manager on a Manager. Follows from the Dependencies table, and holds because the table allows none.

**One gate**: `just check` runs every check listed under Project tasks in the architecture and exits non-zero when any fails; `just test` runs tests only, and `just run` starts the Host in development mode.

**CI runs the gate**: the pull-request workflow runs `just check` on Windows, macOS, and Linux.

**Weekly supply chain**: the supply-chain check also runs on a weekly schedule.

**Named failures**: a failed feature or architecture check prints each offending crate and the rule it broke, and exits non-zero.

## Changes to existing behaviour

None. There is no pinned spec yet.

## Implementation Decisions

- The architecture check reads the Dependencies and Restricted external dependencies tables from `ARCHITECTURE.md` itself, so the documentation and its enforcement cannot drift. The Type column resolves rows such as Client and Host in the restricted table.
- The checks live in the `ci` tool (its own crate outside the workspace, its own lockfile), next to the existing feature checks, and run against `cargo metadata` of the workspace.
- `just check` is the whole gate; the existing `ci` recipe is replaced by it, and the recipe that runs the `ci` tool is named for what it covers (workspace rules), not for features alone.
- Empty crates carry only their manifest, README, and an empty library root (the Host keeps its binary); dependencies are added when a change needs them.
- CI runs the whole of `just check` on each platform, so the gate a contributor runs locally is exactly the gate CI runs.

## Testing

- **Seam: the `ci` tool's checks against fixture workspaces.** Each check is run against small on-disk fixture workspaces, one valid and one violating each Rule, through `just test`. Covers: Required features, Documented features, Dev propagates, Allowed dependencies, Known crates only, Restricted externals, Same-kind isolation, Named failures.
- **Seam: the real workspace.** `just check` passes on the repository. Covers: Every component is a crate, One gate.
- **Not tested automatically**: CI runs the gate, Weekly supply chain. They are exercised by the first pull request and checked in the manual gate.

## Out of Scope

- Any behaviour of the editor: the crates are empty.
- Diagnostics, bundled resources, and release packaging.
- The `review`-marked Rule translation rows; only the enforced rows are automated.
- Render tests in CI (they need a GPU adapter).

## Further Notes

"Contributor" is the engineering role; it is not the domain's Author.
