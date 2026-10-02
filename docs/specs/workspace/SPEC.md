# Workspace

**Commands**: none

## Purpose

The architecture decomposes the editor into components, each its own crate, with rules about which crate may depend on which. This capability makes those rules something a tool checks rather than something a reviewer remembers: every component exists as a crate, one command runs the whole gate, and the gate enforces the workspace rules locally and in CI alike.

## User Stories

1. As a contributor, I can find every component of the architecture as a crate in the workspace, so that a change fills in a crate instead of creating one.
2. As a contributor, I can switch on debug tooling the same way in every crate, because each declares `default` and `dev` features.
3. As a contributor, I can enable `dev` on the Host and have it enabled in every workspace crate it depends on.
4. As a contributor, I can read what a crate's features do in its README, without reading its manifest.
5. As a contributor, I can rely on the gate failing when a crate depends on a workspace crate its row in the Dependencies table does not allow, so that the layering cannot erode unnoticed.
6. As a contributor, I can rely on the gate failing when a workspace crate has no row in the Dependencies table, so that a new component is a deliberate architecture change.
7. As a contributor, I can rely on the gate failing when a restricted external crate is used outside the components allowed it, so that the UI, the render stack, and the asset system stay confined.
8. As a contributor, I can rely on the gate failing when an Engine depends on an Engine, a ResourceAccess on a ResourceAccess, or a Manager on a Manager, so that closed layering holds.
9. As a contributor, I can rely on the gate failing when an Engine defines or uses a message, an event, or an observer, so that Engines stay plain computation that Managers call.
10. As a contributor, I can rely on the gate failing instead of weakening when a dependency table is malformed, so that a typo in the architecture never silently disables a rule.
11. As a contributor, I can run one `just check` for the whole gate, so that I never guess which checks a change must pass.
12. As a contributor, I can see the same gate run on Windows, macOS, and Linux for every pull request, so that platform breakage is found before merging.
13. As a contributor, I can count on the supply-chain check running weekly, so that a newly published advisory is noticed without a pull request.
14. As a contributor, I can receive Dependabot proposals for the workspace, the check tooling, and the CI workflows, so that dependencies do not rot.
15. As a contributor, I can read which crate broke which rule when a check fails, so that I can fix it without reading the checker.
16. As a contributor, I can rely on the gate failing when the workspace's bundle marker names a version other than the crates', so that an editor built from the workspace never warns about its own Bundled Files.

## Rules

**Every component is a crate**: the workspace contains exactly one `drs-<component>` crate for each row of the Dependencies table, and none besides.

**Required features**: every workspace crate declares `default` and `dev`.

**Documented features**: every workspace crate has a README, and every feature it declares other than `default` and `dev` appears in that README as a code-formatted name.

**Dev propagates**: a crate's `dev` feature enables `dev` on every workspace crate it depends on.

**Allowed dependencies**: a workspace crate depends only on the workspace crates listed in its row of the Dependencies table.

**Known crates only**: a workspace crate with no row in the Dependencies table fails the architecture check.

**Restricted externals**: an external crate named in the Restricted external dependencies table is a dependency only of the crates or component types listed for it.

**Well-formed tables**: a Dependencies or Restricted external dependencies row with the wrong number of cells, an unknown component type, a duplicate component, or a name that is neither a component nor a component type fails the architecture check instead of weakening it.

**Same-kind isolation**: no Engine depends on an Engine, no ResourceAccess on a ResourceAccess, no Manager on a Manager. Follows from the Dependencies table, and holds because the table allows none.

**Bundle marker names the version**: the marker file `bundle/dungeon-rs.bundle`, which marks the workspace's bundle directory as the editor's, names the version every workspace crate has, so an editor built from the workspace finds its Bundled Files without a warning.

**Engines raise no events**: the sources of a crate the Dependencies table types as an Engine neither derive a message, event, or entity event nor read, write, or observe one; a mention in a comment, or a longer name that merely contains one of those names, does not count.

**One gate**: `just check` runs every check listed under Project tasks in the architecture and exits non-zero when any fails; `just test` runs tests only, and `just run` starts the Host in development mode.

**CI runs the gate**: the CI workflow runs `just check` on Windows, macOS, and Linux for every pull request and every push to the default branch.

**Weekly supply chain**: the supply-chain check also runs on a weekly schedule.

**Named failures**: a failed feature or architecture check prints each offending crate, the rule it broke, and what is wrong, and exits non-zero.

## Implementation Decisions

- The workspace rules are checked by the `ci` tool: its own crate outside the workspace, with its own lockfile, run through the `workspace` recipe of the `justfile`. It has one subcommand per check and an `all` subcommand that runs every check, so one run reports everything that is wrong.
- Every check runs against `cargo metadata` of the workspace, which the tool locates from its own position rather than from the directory it is run in, so it reads manifests, not source; the Engine-events check, which reads the Engine crates' sources, and the bundle marker check, which reads the marker file, are the exceptions.
- The architecture check reads the Dependencies and Restricted external dependencies tables from the architecture document itself, so the documentation and its enforcement cannot drift. The Type column resolves rows such as Client and Host in the restricted table. A malformed table is an error, never a weaker check.
- Same-kind isolation is checked on its own, in addition to the allowed dependencies, so a table that wrongly allowed an Engine to depend on an Engine is still caught.
- The Engine-events check is the one check that reads source rather than manifests: every Rust file of every Engine crate, with comments stripped, is searched for the derives that define a message or an event and the names that read, write, or observe one, and each hit is reported with its file and line. It is the Rule translation row marked `enforced`.
- A violation is one crate breaking one rule, with a detail of what is wrong. All violations are rendered as a single table of crate, rule, and problem on standard error, and the tool exits non-zero when there is at least one.
- The checks are tested against fixture workspaces: small throwaway workspaces written to a temporary directory and read back through `cargo metadata`, one passing and one violating each rule.
- Empty crates carry only their manifest, README, and an empty library root; the Host keeps its binary. Dependencies are added by the changes that need them.
- `just check` is the whole gate: formatting (the workspace, the `ci` tool, and the TOML files), lints, typos, supply chain and licences, the workspace rules, commit messages, tests (including the `ci` tool's own), a build on the minimum supported Rust version, and a warning-free docs build. On Linux and macOS the tests and lints use the `fast` profile; on Windows they do not. _Why_: the `fast` profile breaks on Windows under linker limits.
- The commit-message check lints every commit since `origin/master`, or since the first commit while there is no remote.
- The CI workflow runs the whole of `just check` on each platform, so the gate a contributor runs locally is exactly the gate CI runs. It checks out a pull request at its own head so that the commit messages linted are the pull request's own, and denies warnings through `CARGO_BUILD_WARNINGS` rather than `RUSTFLAGS`, which would invalidate the build cache.
- The supply-chain workflow runs cargo-deny every Monday and on demand. Dependabot proposes weekly updates for the workspace, the `ci` tool, and the GitHub Actions used by the workflows.

## Test seams

- **Every component is a crate**: `tools/ci/src/architecture.rs::a_missing_component_is_reported`, `tools/ci/src/architecture.rs::a_crate_without_a_row_is_reported`, `tools/ci/src/architecture.rs::a_workspace_that_follows_the_tables_passes`
- **Required features**: `tools/ci/src/required_features.rs::a_crate_with_both_required_features_passes`, `tools/ci/src/required_features.rs::a_crate_without_dev_is_named_with_the_missing_feature`, `tools/ci/src/required_features.rs::a_crate_without_default_is_reported`
- **Documented features**: `tools/ci/src/documented_features.rs::a_readme_that_mentions_every_extra_feature_passes`, `tools/ci/src/documented_features.rs::an_undocumented_feature_is_named`, `tools/ci/src/documented_features.rs::a_crate_without_a_readme_is_reported`, `tools/ci/src/documented_features.rs::a_feature_name_inside_another_word_does_not_count_as_documented`
- **Dev propagates**: `tools/ci/src/workspace_features.rs::dev_enabling_dev_on_every_workspace_dependency_passes`, `tools/ci/src/workspace_features.rs::dev_that_skips_a_workspace_dependency_is_named`, `tools/ci/src/workspace_features.rs::external_dependencies_need_no_propagation`
- **Allowed dependencies**: `tools/ci/src/architecture.rs::a_workspace_that_follows_the_tables_passes`, `tools/ci/src/architecture.rs::a_dependency_the_row_does_not_allow_is_named`
- **Known crates only**: `tools/ci/src/architecture.rs::a_crate_without_a_row_is_reported`
- **Restricted externals**: `tools/ci/src/architecture.rs::a_restricted_external_in_an_allowed_component_type_passes`, `tools/ci/src/architecture.rs::a_restricted_external_in_an_allowed_crate_passes`, `tools/ci/src/architecture.rs::a_restricted_external_outside_its_allowed_components_is_named`, `tools/ci/src/architecture.rs::an_unrestricted_external_is_free_to_use`, `tools/ci/src/architecture.rs::the_host_type_resolves_a_restricted_external`
- **Well-formed tables**: `tools/ci/src/architecture.rs::the_tables_are_read_from_the_markdown`, `tools/ci/src/architecture.rs::a_document_without_the_tables_is_an_error`, `tools/ci/src/architecture.rs::a_row_with_the_wrong_number_of_cells_is_an_error`, `tools/ci/src/architecture.rs::an_unknown_component_type_is_an_error`, `tools/ci/src/architecture.rs::a_duplicate_row_is_an_error`, `tools/ci/src/architecture.rs::a_dependency_on_an_unlisted_component_is_an_error`, `tools/ci/src/architecture.rs::a_restriction_naming_an_unknown_type_is_an_error`
- **Same-kind isolation**: `tools/ci/src/architecture.rs::an_engine_depending_on_an_engine_breaks_same_kind_isolation`, `tools/ci/src/architecture.rs::a_resource_access_depending_on_a_resource_access_breaks_same_kind_isolation`, `tools/ci/src/architecture.rs::a_manager_depending_on_a_manager_breaks_same_kind_isolation`
- **Engines raise no events**: `tools/ci/src/engine_events.rs::an_engine_of_plain_systems_passes`, `tools/ci/src/engine_events.rs::an_engine_defining_a_message_is_named_with_the_line`, `tools/ci/src/engine_events.rs::an_engine_defining_an_event_is_named`, `tools/ci/src/engine_events.rs::an_engine_reading_or_writing_messages_is_named`, `tools/ci/src/engine_events.rs::an_engine_observing_is_named`, `tools/ci/src/engine_events.rs::a_mention_in_a_comment_is_not_a_use`, `tools/ci/src/engine_events.rs::a_longer_name_sharing_the_letters_is_not_a_use`, `tools/ci/src/engine_events.rs::a_manager_may_use_messages`
- **Bundle marker names the version**: `tools/ci/src/bundle_marker.rs::a_marker_naming_the_workspace_version_passes`, `tools/ci/src/bundle_marker.rs::a_marker_naming_another_version_is_named_with_both`, `tools/ci/src/bundle_marker.rs::a_missing_marker_is_reported`
- **One gate**: no unit test; the seam is the `check` recipe of the `justfile`, exercised by every run of the gate.
- **CI runs the gate**: no unit test; the seam is the CI workflow, exercised by every pull request.
- **Weekly supply chain**: no unit test; the seam is the supply-chain workflow, exercised by its schedule.
- **Named failures**: `tools/ci/src/violation.rs::every_violation_names_its_crate_rule_and_problem`, `tools/ci/src/required_features.rs::a_crate_without_dev_is_named_with_the_missing_feature`, `tools/ci/src/documented_features.rs::an_undocumented_feature_is_named`, `tools/ci/src/workspace_features.rs::dev_that_skips_a_workspace_dependency_is_named`, `tools/ci/src/architecture.rs::a_dependency_the_row_does_not_allow_is_named`, `tools/ci/src/architecture.rs::a_restricted_external_outside_its_allowed_components_is_named`

## Not supported

- The rows of the Rule translation table marked `review` are not checked by the tool; the standards review is their only guard.
- The checks read manifests only, except the Engine-events check, which reads source, and the bundle marker check, which reads the marker file. A dependency reached through another crate's re-exports, or any other rule about what source code does, is outside them.

## Notes

"Contributor" is the engineering role; it is not the domain's Author.
