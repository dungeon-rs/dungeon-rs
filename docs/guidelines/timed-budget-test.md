# Timed budget test

**Use when**: a test bounds how long the code takes, because the architecture or a spec gives it a time budget (per keystroke, per frame, per build). **Not when**: a test only waits for background work under a patience limit (`PATIENCE` in the thumbnail tests), which says nothing about speed.
**Exemplar**: `crates/drs-catalog-engine/tests/search.rs`

## Rules

- Time the work at the seam the budget names, over data the test generates in the shape and size of the case the architecture measured (400,000 vendor-shaped names), with the inputs that take the slow paths mixed in (accented names, composed and decomposed, beside ASCII ones). Assert the data's size before timing, so a generator that shrinks cannot pass the bound.
- Measure through a `fastest(runs, work)` helper: `Instant::now()` around the work alone, the shortest of a few runs kept, and what the last run gave returned, so the test checks the outcome and the work cannot be optimised away. Bound the fastest run, never a mean.
- Work done once behind a cache (the order the library search works out at its first search) is timed in a run that starts from a fresh value; the steady state is timed separately over the warm one.
- Each bound is one `assert!` whose message holds the measured value, after an `eprintln!` of every measurement, so `cargo nextest run --no-capture` shows the margin.
- The bound is for the build the Author runs (`dev`: our crates at opt-level 1, dependencies at 3). Every crate on the timed path that the `fast` profile leaves unoptimised gets a `[profile.fast.package.<crate>]` override in `Cargo.toml` at the level `dev` gives it, and is named in the Profiles bullet of `docs/architecture/ARCHITECTURE.md`.
- The test runs alone: add it to the filter of the `timed` test group's override in `.config/nextest.toml`, which runs one such test at a time and only when every other test slot is free (`threads-required = 'num-test-threads'`).

## Example

```rust
/// The fastest of `runs` runs of `work`, and what the last run gave: the least disturbed run
/// is the one that says what the code costs.
fn fastest<T>(runs: usize, mut work: impl FnMut() -> T) -> (Duration, T) {
    let mut best = Duration::MAX;
    let mut last = None;
    for _ in 0..runs {
        let start = Instant::now();
        let outcome = work();
        best = best.min(start.elapsed());
        last = Some(outcome);
    }
    (best, last.expect("at least one run"))
}

/// Over 400,000 Assets in the test build, building the search and answering the first text
/// takes under 1 second and answering each text after it under 50 milliseconds.
#[test]
fn answered_within_a_keystroke() {
    let library = vendor_library();
    let folders: Vec<&Folder> = library.iter().collect();
    let total: usize = library.iter().map(|folder| folder.assets.len()).sum();
    assert!(total >= 399_000, "{total} Assets");

    let (built, (mut search, first)) = fastest(3, || {
        let mut search = library_of(&folders);
        let first = answer(&mut search, "a", &folders);
        (search, first)
    });
    eprintln!(
        "built the search of {total} Assets and matched {} of them in {built:?}",
        first.assets.len()
    );
    assert!(
        built < Duration::from_secs(1),
        "built and first answered in {built:?}"
    );

    let narrow = library[0].assets[1234].name.to_lowercase();
    for text in [
        narrow.as_str(),
        "a",
        "oak table",
        "ÉPÉE",
        "adventures/crâne_pack chair",
    ] {
        let (answered, found) = fastest(5, || answer(&mut search, text, &folders));
        eprintln!(
            "{text:?} matched {} Assets in {answered:?}",
            found.assets.len()
        );
        assert!(!found.assets.is_empty(), "{text:?} matches something");
        assert!(
            answered < Duration::from_millis(50),
            "{text:?} answered in {answered:?}"
        );
    }
}
```

## Pitfalls

- A bound met only by ASCII data says nothing of the Unicode path, which an ASCII shortcut skips entirely; unoptimised, folding alone took the library search's build past two seconds.
- A test thread beside the timed one takes a core and the caches from it: without the `timed` group the bound fails on a busy machine with nothing slower.
- The doc comment states the budget with its data size and build, as the spec's Rule does; "fast enough" is no bound.
