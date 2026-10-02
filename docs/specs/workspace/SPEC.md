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
17. As a contributor, I can rely on the gate failing when a guideline's Example is no longer taken from its Exemplar, so that a guideline never teaches code the workspace has moved away from.
18. As a contributor, I can rely on the gate failing when a crate uses a Bevy crate the architecture neither lists as narrow nor restricts, so that every new Bevy crate is a deliberate decision.
19. As a contributor, I can rely on the gate failing when a WGSL Shader a crate bundles does not parse or validate, naming the file and the line, so that a broken Shader is found without a GPU rather than when the editor first draws with it.

## Rules

**Every component is a crate**: the workspace contains exactly one `drs-<component>` crate for each row of the Dependencies table, and none besides.

**Required features**: every workspace crate declares `default` and `dev`.

**Documented features**: every workspace crate has a README, and every feature it declares other than `default` and `dev` appears in that README as a code-formatted name.

**Dev propagates**: a crate's `dev` feature enables `dev` on every workspace crate it depends on.

**Allowed dependencies**: a workspace crate depends only on the workspace crates listed in its row of the Dependencies table.

**Known crates only**: a workspace crate with no row in the Dependencies table fails the architecture check.

**Restricted externals**: an external crate named in the Restricted external dependencies table is a dependency only of the crates or component types listed for it.

**Known Bevy crates**: every Bevy crate (`bevy` or a `bevy_` crate) a workspace crate depends on, dev-dependencies included, is one of the narrow crates the Framework boundary bullet of the architecture lists in parentheses or has a row in the Restricted external dependencies table; a missing bullet, a bullet without the list, or a list item that is not a code-formatted Bevy crate name fails the check instead of weakening it.

**Well-formed tables**: a Dependencies or Restricted external dependencies row with the wrong number of cells, an unknown component type, a duplicate component, or a name that is neither a component nor a component type fails the architecture check instead of weakening it.

**Same-kind isolation**: no Engine depends on an Engine, no ResourceAccess on a ResourceAccess, no Manager on a Manager. Follows from the Dependencies table, and holds because the table allows none.

**Bundle marker names the version**: the marker file `bundle/dungeon-rs.bundle`, which marks the workspace's bundle directory as the editor's, names the version every workspace crate has, so an editor built from the workspace finds its Bundled Files without a warning.

**Guideline examples come from their exemplars**: every guideline in `docs/guidelines` other than the index names an Exemplar file and has an Example whose lines, blank lines aside and each compared without its indentation, all appear in that file in the same order, with any lines between; a guideline that names no Exemplar, has no Example, names an Exemplar that is missing or unreadable, or holds a line not found in order fails the check with that guideline named.

**Engines raise no events**: the sources of a crate the Dependencies table types as an Engine neither derive a message, event, or entity event nor read, write, or observe one; a mention in a comment, or a longer name that merely contains one of those names, does not count.

**Shaders validate**: every `.wgsl` file directly in a directory named `shaders` anywhere under the workspace's `crates` directory parses as WGSL and passes naga's validation, the one wgpu runs before it compiles a Shader; a file that does not fails the check with the file named, and with the line where naga gives one; a workspace with no such file passes.

**One gate**: `just check` runs every check listed under Project tasks in the architecture and exits non-zero when any fails; `just test` runs tests only, and `just run` starts the Host in development mode.

**CI runs the gate**: the CI workflow runs `just check` on Windows, macOS, and Linux for every pull request and every push to the default branch.

**Weekly supply chain**: the supply-chain check also runs on a weekly schedule.

**Named failures**: a failed feature or architecture check prints each offending crate, the rule it broke, and what is wrong, and exits non-zero.

## Implementation Decisions

- The workspace rules are checked by the `ci` tool: its own crate outside the workspace, with its own lockfile, run through the `workspace` recipe of the `justfile`. It has one subcommand per check and an `all` subcommand that runs every check, so one run reports everything that is wrong.
- Every check runs against `cargo metadata` of the workspace, which the tool locates from its own position rather than from the directory it is run in, so it reads manifests, not source; the Engine-events check, which reads the Engine crates' sources, the bundle marker check, which reads the marker file, the guideline-examples check, which reads the guidelines and their Exemplar files, and the WGSL check, which reads the Shader files, are the exceptions.
- The architecture check reads the Dependencies and Restricted external dependencies tables from the architecture document itself, so the documentation and its enforcement cannot drift. The Type column resolves rows such as Client and Host in the restricted table. A malformed table is an error, never a weaker check.
- The Bevy-crates check reads the narrow crates from the first parenthesised list of the architecture's Framework boundary bullet and the Restricted external dependencies table from the same document, and names each crate using any other `bevy` or `bevy_` crate with that crate. _Why_: Bevy is split into many crates, and one added without a row would otherwise escape every restriction.
- Same-kind isolation is checked on its own, in addition to the allowed dependencies, so a table that wrongly allowed an Engine to depend on an Engine is still caught.
- The Engine-events check reads source rather than manifests: every Rust file of every Engine crate, with comments stripped, is searched for the derives that define a message or an event and the names that read, write, or observe one, and each hit is reported with its file and line. It is the Rule translation row marked `enforced`.
- The guideline-examples check takes each guideline's Exemplar from its `**Exemplar**:` line and its Example from the first fenced block under its Example heading, and names the first line of the Example it cannot find in order. A workspace without a guidelines directory passes it.
- The WGSL check finds the Shaders under the workspace's `crates` directory itself, directly in every directory named `shaders`, and parses and validates each with naga, the WGSL front end and validator wgpu uses, built into the `ci` tool alone, so the check needs neither a GPU nor Bevy; it names the line of a parse failure and of a validation failure where naga locates one.
- A violation is one crate, or for the guideline-examples check one guideline and for the WGSL check one Shader file, breaking one rule, with a detail of what is wrong. All violations are rendered as a single table of crate or guideline, rule, and problem on standard error, and the tool exits non-zero when there is at least one.
- The checks are tested against fixture workspaces: small throwaway workspaces written to a temporary directory and read back through `cargo metadata`, one passing and one violating each rule.
- Empty crates carry only their manifest, README, and an empty library root; the Host keeps its binary. Dependencies are added by the changes that need them.
- `just check` is the whole gate: formatting (the workspace, the `ci` tool, and the TOML files), lints, typos, supply chain and licences, the workspace rules (including the guideline examples), commit messages, tests (including the `ci` tool's own), a build on the minimum supported Rust version, and a warning-free docs build. On Linux and macOS the tests and lints use the `fast` profile; on Windows they do not. _Why_: the `fast` profile breaks on Windows under linker limits.
- The commit-message check lints every commit since `origin/master`, or since the first commit while there is no remote.
- The CI workflow runs the whole of `just check` on each platform, so the gate a contributor runs locally is exactly the gate CI runs. It checks out a pull request at its own head so that the commit messages linted are the pull request's own, and denies warnings through `CARGO_BUILD_WARNINGS` rather than `RUSTFLAGS`, which would invalidate the build cache.
- The supply-chain workflow runs cargo-deny every Monday and on demand. Dependabot proposes weekly updates for the workspace, the `ci` tool, and the GitHub Actions used by the workflows.

## Test seams

- **Every component is a crate**: `tools/ci/src/architecture.rs::tests::a_missing_component_is_reported`, `tools/ci/src/architecture.rs::tests::a_crate_without_a_row_is_reported`, `tools/ci/src/architecture.rs::tests::a_workspace_that_follows_the_tables_passes`
- **Required features**: `tools/ci/src/required_features.rs::tests::a_crate_with_both_required_features_passes`, `tools/ci/src/required_features.rs::tests::a_crate_without_dev_is_named_with_the_missing_feature`, `tools/ci/src/required_features.rs::tests::a_crate_without_default_is_reported`
- **Documented features**: `tools/ci/src/documented_features.rs::tests::a_readme_that_mentions_every_extra_feature_passes`, `tools/ci/src/documented_features.rs::tests::an_undocumented_feature_is_named`, `tools/ci/src/documented_features.rs::tests::a_crate_without_a_readme_is_reported`, `tools/ci/src/documented_features.rs::tests::a_feature_name_inside_another_word_does_not_count_as_documented`
- **Dev propagates**: `tools/ci/src/workspace_features.rs::tests::dev_enabling_dev_on_every_workspace_dependency_passes`, `tools/ci/src/workspace_features.rs::tests::dev_that_skips_a_workspace_dependency_is_named`, `tools/ci/src/workspace_features.rs::tests::external_dependencies_need_no_propagation`
- **Allowed dependencies**: `tools/ci/src/architecture.rs::tests::a_workspace_that_follows_the_tables_passes`, `tools/ci/src/architecture.rs::tests::a_dependency_the_row_does_not_allow_is_named`
- **Known crates only**: `tools/ci/src/architecture.rs::tests::a_crate_without_a_row_is_reported`
- **Restricted externals**: `tools/ci/src/architecture.rs::tests::a_restricted_external_in_an_allowed_component_type_passes`, `tools/ci/src/architecture.rs::tests::a_restricted_external_in_an_allowed_crate_passes`, `tools/ci/src/architecture.rs::tests::a_restricted_external_outside_its_allowed_components_is_named`, `tools/ci/src/architecture.rs::tests::an_unrestricted_external_is_free_to_use`, `tools/ci/src/architecture.rs::tests::the_host_type_resolves_a_restricted_external`
- **Known Bevy crates**: `tools/ci/src/bevy_crates.rs::tests::the_narrow_crates_are_read_from_the_bullet`, `tools/ci/src/bevy_crates.rs::tests::narrow_and_restricted_bevy_crates_pass`, `tools/ci/src/bevy_crates.rs::tests::a_bevy_crate_nobody_chose_is_named`, `tools/ci/src/bevy_crates.rs::tests::a_bevy_crate_only_tests_use_is_named`, `tools/ci/src/bevy_crates.rs::tests::a_crate_outside_bevy_is_free_to_use`, `tools/ci/src/bevy_crates.rs::tests::a_document_without_the_bullet_is_an_error`, `tools/ci/src/bevy_crates.rs::tests::a_bullet_without_a_list_is_an_error`, `tools/ci/src/bevy_crates.rs::tests::a_list_item_outside_bevy_is_an_error`, `tools/ci/src/bevy_crates.rs::tests::a_list_item_that_is_not_a_crate_name_is_an_error`
- **Well-formed tables**: `tools/ci/src/architecture.rs::tests::the_tables_are_read_from_the_markdown`, `tools/ci/src/architecture.rs::tests::a_document_without_the_tables_is_an_error`, `tools/ci/src/architecture.rs::tests::a_row_with_the_wrong_number_of_cells_is_an_error`, `tools/ci/src/architecture.rs::tests::an_unknown_component_type_is_an_error`, `tools/ci/src/architecture.rs::tests::a_duplicate_row_is_an_error`, `tools/ci/src/architecture.rs::tests::a_dependency_on_an_unlisted_component_is_an_error`, `tools/ci/src/architecture.rs::tests::a_restriction_naming_an_unknown_type_is_an_error`
- **Same-kind isolation**: `tools/ci/src/architecture.rs::tests::an_engine_depending_on_an_engine_breaks_same_kind_isolation`, `tools/ci/src/architecture.rs::tests::a_resource_access_depending_on_a_resource_access_breaks_same_kind_isolation`, `tools/ci/src/architecture.rs::tests::a_manager_depending_on_a_manager_breaks_same_kind_isolation`
- **Guideline examples come from their exemplars**: `tools/ci/src/guideline_examples.rs::tests::an_example_trimmed_from_its_exemplar_passes`, `tools/ci/src/guideline_examples.rs::tests::a_line_not_in_the_exemplar_is_named`, `tools/ci/src/guideline_examples.rs::tests::lines_in_another_order_are_named`, `tools/ci/src/guideline_examples.rs::tests::a_missing_exemplar_is_reported`, `tools/ci/src/guideline_examples.rs::tests::a_guideline_without_an_exemplar_or_an_example_is_reported`, `tools/ci/src/guideline_examples.rs::tests::a_workspace_without_guidelines_passes`
- **Engines raise no events**: `tools/ci/src/engine_events.rs::tests::an_engine_of_plain_systems_passes`, `tools/ci/src/engine_events.rs::tests::an_engine_defining_a_message_is_named_with_the_line`, `tools/ci/src/engine_events.rs::tests::an_engine_defining_an_event_is_named`, `tools/ci/src/engine_events.rs::tests::an_engine_reading_or_writing_messages_is_named`, `tools/ci/src/engine_events.rs::tests::an_engine_observing_is_named`, `tools/ci/src/engine_events.rs::tests::a_mention_in_a_comment_is_not_a_use`, `tools/ci/src/engine_events.rs::tests::a_longer_name_sharing_the_letters_is_not_a_use`, `tools/ci/src/engine_events.rs::tests::a_manager_may_use_messages`
- **Bundle marker names the version**: `tools/ci/src/bundle_marker.rs::tests::a_marker_naming_the_workspace_version_passes`, `tools/ci/src/bundle_marker.rs::tests::a_marker_naming_another_version_is_named_with_both`, `tools/ci/src/bundle_marker.rs::tests::a_missing_marker_is_reported`
- **Shaders validate**: `tools/ci/src/wgsl_shaders.rs::tests::a_valid_shader_passes`, `tools/ci/src/wgsl_shaders.rs::tests::a_shader_that_does_not_parse_is_named_with_its_line`, `tools/ci/src/wgsl_shaders.rs::tests::a_shader_that_does_not_validate_is_named`, `tools/ci/src/wgsl_shaders.rs::tests::only_wgsl_files_in_a_shaders_directory_are_checked`, `tools/ci/src/wgsl_shaders.rs::tests::a_workspace_without_shaders_passes`
- **One gate**: no unit test; the seam is the `check` recipe of the `justfile`, exercised by every run of the gate.
- **CI runs the gate**: no unit test; the seam is the CI workflow, exercised by every pull request.
- **Weekly supply chain**: no unit test; the seam is the supply-chain workflow, exercised by its schedule.
- **Named failures**: `tools/ci/src/violation.rs::tests::every_violation_names_its_crate_rule_and_problem`, `tools/ci/src/required_features.rs::tests::a_crate_without_dev_is_named_with_the_missing_feature`, `tools/ci/src/documented_features.rs::tests::an_undocumented_feature_is_named`, `tools/ci/src/workspace_features.rs::tests::dev_that_skips_a_workspace_dependency_is_named`, `tools/ci/src/architecture.rs::tests::a_dependency_the_row_does_not_allow_is_named`, `tools/ci/src/architecture.rs::tests::a_restricted_external_outside_its_allowed_components_is_named`

## Not supported

- The rows of the Rule translation table marked `review` are not checked by the tool; the standards review is their only guard.
- The checks read manifests only, except the Engine-events check, which reads source, the bundle marker check, which reads the marker file, the guideline-examples check, which reads the guidelines and their Exemplar files, and the WGSL check, which reads the Shader files; a Shader that validates may still disagree with the Bevy types it mirrors, which no check compares. A dependency reached through another crate's re-exports, or any other rule about what source code does, is outside them.

## Notes

- "Contributor" is the engineering role; it is not the domain's Author.
- Three Rules (One gate, CI runs the gate, Weekly supply chain) have no test, against the requirement that every Rule has one. They are facts of the `justfile` and the CI workflows rather than of the `ci` tool, and the accepted deviation is that the gate itself is their seam: every run of `just check`, every pull request, and the weekly schedule exercise them.
