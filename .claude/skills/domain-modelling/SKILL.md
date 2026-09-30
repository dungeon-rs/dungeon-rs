---
name: domain-modelling
description: Build and sharpen the project's domain model under docs/domain/. Use when discussing domain terminology, when a term conflicts with the glossary, or when writing or editing any file in docs/domain/.
---

<!-- Adapted from mattpocock/skills (MIT). -->

# Domain Modelling

Actively build and sharpen the domain model: challenge terms, invent edge-case scenarios, and write the model down the moment a piece of it crystallizes. (Merely *reading* `docs/domain/` for vocabulary is not this skill; any skill can do that. This skill is for when the model is changing.)

The domain describes **what exists and what must always be true**. It is technology-agnostic: no frameworks, languages, libraries, framework-specific structures, file formats-as-bytes, or UI widgets. What happens when the user does X belongs in specs; product vision and principles belong in `docs/PRODUCT.md`. Use the format in [DOMAIN-FORMAT.md](./DOMAIN-FORMAT.md).

## During the session

### Challenge against the model

When the user uses a term that conflicts with the existing language, call it out immediately. "The glossary defines 'Order' as X, but you seem to mean Y. Which is it?" Also call out any term listed under another term's `_Avoid_`, and any term that clashes with an industry-standard meaning.

### Sharpen fuzzy language

When the user uses vague or overloaded terms, propose a precise canonical term. "You're saying 'item': do you mean a Product or any Line Item? Those are different things."

If the user says a term keeps its industry-standard meaning, don't invent a bespoke definition: write a one-line definition and mark it "Industry-standard meaning."

### Discuss concrete scenarios

When relationships or invariants are discussed, stress-test them with specific scenarios that probe the boundaries. "A Line Item refers to a Product, then the Product is discontinued. What happens to the Line Item?"

Where the product is shared or concurrent, include scenarios with **two parties on different setups**: different devices, platforms, versions of the same content, or partial copies of it. Single-party scenarios miss the failures that only show up when work is shared.

### Cross-reference with specs and code

When the user states how something works and specs or code exist, check whether they agree. Surface contradictions; don't resolve them silently.

### Write inline

When a term or invariant is resolved, write it into `docs/domain/` right there. Don't batch. Create files lazily: only when there is something to write.

Spec-level decisions that surface along the way (UI behaviour, thresholds, formats, performance targets) go to `docs/specs/OPEN-QUESTIONS.md`, not the domain.

## The domain is meant to be stable

A domain that shifts a lot means it was modelled wrong. Adding to the model is normal; **changing or removing** an existing definition or invariant is a domain change and gets treated as one:

1. Say plainly that this is a domain change, quoting old and new side by side:
   > **Domain change: {term}.** The domain says: *"{old statement}"*. This makes it: *"{new statement}"*.
2. Find every spec and domain file that references it (dispatch a sub-agent for the search) and list them.
3. Get explicit confirmation before editing. An answer from the user that directly states the new rule counts; say that you are treating it as confirmation.

The domain states what is true now. Do not keep history, changelogs, or "previously…" notes in it: history lives in version control. Rationale is allowed, but only when a future reader would otherwise find a rule surprising, as a single `_Why_:` line under it.
