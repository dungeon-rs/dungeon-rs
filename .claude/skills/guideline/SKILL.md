---
name: guideline
description: Write or update a guideline for a recurring building block, extracted from real code.
disable-model-invocation: true
argument-hint: "<building block>"
---

A guideline says **how to build one kind of building block** in this project, with an example taken from real code, so nobody re-derives it. Guidelines live in `docs/guidelines/`, one file per building block, indexed in `docs/guidelines/INDEX.md`. Format: [GUIDELINE-FORMAT.md](./GUIDELINE-FORMAT.md).

Invoke it as `/guideline <building block>`.

## When a guideline exists

- The **second time** a kind of building block is written, or the **first time** when the architecture guarantees it will repeat (every Command is undoable; every module is wired in the same way).
- The standards review flags the second unguided instance; `implement` then offers this skill before landing.

## Rules

- **Checks before prose.** For every rule, ask first: can a tool check this? If so, put it in the linter or formatter configuration or in the project's `just check` instead, and let the guideline at most name the check. Only what no tool can check is written as a rule.
- **Extracted, not invented.** A guideline describes code that exists and works; write it from that code.
- **How, never what or why.** Don't restate architecture rules (link to `docs/architecture/ARCHITECTURE.md`), behaviour (specs), or domain meaning (`docs/domain/`).
- **Current state.** A guideline changes in the same commit as the pattern it describes. No history.
- **Examples are real.** An example is a trimmed excerpt of the exemplar, so it stays true to code that runs. The standards review flags an example that has drifted from its exemplar.

## Write

1. Read `docs/guidelines/INDEX.md` and any guideline for the same building block.
2. Find the real instances of the building block in the code (dispatch a sub-agent for the search). Pick the clearest as the exemplar.
3. Triage each candidate rule: checkable → into tool configuration; otherwise → the guideline's Rules (3–7).
4. Write the guideline per the format, with a minimal complete example derived from the exemplar.
5. Add or update its line in `INDEX.md`.
6. Show the user the guideline and any rules moved into tool configuration; commit only what they confirm.
