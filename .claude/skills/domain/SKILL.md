---
name: domain
description: Build or extend the project's domain model from a brain-dump, then interview to fill gaps, resolve ambiguities, and surface contradictions.
disable-model-invocation: true
argument-hint: "[brain-dump]"
---

Build the domain model in `docs/domain/` from what the user knows. Five phases, in order. The model is settled in two passes: **structure** (the things: terms and invariants) before **behaviour** (commands, events, workflows). Don't draft behaviour until the structure pass is done.

## 1. Intake

Read `docs/domain/` and `docs/PRODUCT.md` if they exist, including `docs/domain/OPEN-QUESTIONS.md`. If questions are open from an earlier session, offer to resume from them before taking a new dump.

Ask the user to dump everything they know about the domain (or the part they want to work on) in whatever order and shape it comes, starting with what the product is for and who uses it. Don't interrupt, correct, or ask questions during the dump. If the user already provided the dump as arguments, skip the ask. If `docs/PRODUCT.md` doesn't exist and the dump holds no product vision, ask for it once the dump is done.

Then ask one question: **is there an existing product that serves as a functional baseline?** If so, dispatch a background sub-agent to report a factual baseline of it in its reply: what the user works with, how each thing is created and edited, known pain points, with sources. While it runs, do the analysis below, but hold the first grilling round until it reports. When it reports, write the baseline into `docs/PRODUCT.md` (its name, and each pain point that matters as one line; see [DOMAIN-FORMAT.md](../domain-modelling/DOMAIN-FORMAT.md#productmd)) and fold its vocabulary into the domain.

## 2. Analysis

**Sort the dump first.** A dump mixes several kinds of statement. Put each where it belongs, and tell the user where each went:

| Kind | Example | Goes to |
|---|---|---|
| Domain | "an order is just a list of line items" | the draft model |
| Product vision or principle | "the tool gets out of your way" | `docs/PRODUCT.md` (see [DOMAIN-FORMAT.md](../domain-modelling/DOMAIN-FORMAT.md#productmd)) |
| Quality requirement | "must stay fast with 400,000 orders" | `docs/PRODUCT.md` as a principle if it's a lasting value; a `needs-decision` issue if it's a concrete target |
| Spec concern | "the export dialog should remember the last format" | a `needs-decision` issue |

Then turn the domain statements into a **draft model** and a set of **findings**. Don't write domain files yet.

- **Draft model**: candidate contexts, terms, and invariants, in the shape of [DOMAIN-FORMAT.md](../domain-modelling/DOMAIN-FORMAT.md). Structure only; no commands, events, or workflows yet. Mark anything you inferred rather than read from the dump.
- **Findings**, each tagged:
  - `gap`: something the model needs that the dump doesn't say
  - `ambiguity`: a statement with more than one reasonable reading, or a concept referred to by several names
  - `contradiction`: two statements that can't both be true, including conflicts with the existing `docs/domain/`
- **Baseline clashes**: when the baseline's vocabulary is adopted, check each of its terms against the glossary and industry-standard meanings. A baseline term that means something else here is a `contradiction` finding (e.g. the baseline calls a kind of entry a "Transaction" while the domain uses Transaction in its industry-standard database sense).

Show the user the draft model compactly, then the count of findings per tag. Write the findings to `docs/domain/OPEN-QUESTIONS.md`.

## 3. Structure pass

Call the Skill tool for "domain-modelling", then for "grilling". Hand grilling the findings as its initial frontier and `docs/domain/OPEN-QUESTIONS.md` as its questions file. Contradictions go first: they usually settle several gaps at once.

As each answer lands, write the resolved piece into `docs/domain/` right away, following domain-modelling.

- **Research, not interrogation.** When the user can't answer from what they know ("research this", "this is the hardest question"), don't push. Dispatch a background sub-agent to research it against a **scenario suite**: 8–12 concrete situations any answer must survive, including at least one with two parties on different setups where the product is shared or concurrent. Hold the next round until it reports, and do the non-interactive work meanwhile.
- **Sort research results.** A research reply mixes domain decisions with spec-level ones. Bring only the domain decisions into the next round; park the rest as a GitHub issue (a Task, or a Feature for a new capability) labelled `needs-decision`, with its `area:` label when the capability is known.
- **Split into contexts** when `DOMAIN.md` holds roughly 25 terms or no longer fits a screen or two. Propose the split as a round question.

The structure pass ends when the user confirms the things are settled.

## 4. Behaviour pass

Commands, events, and workflows rarely come up on their own: interviews gravitate to nouns. Draft them from the settled structure and the user's answers, one context at a time, and put them to the user as a grilling round:

- **Commands**: the user intentions each context supports. Every "nothing fixed at creation"-style invariant implies an edit command; every relationship implies something that creates and removes it.
- **Events**: only those another context reacts to, with the reaction.
- **Workflows**: the few end-to-end journeys the user described or implied, one for each thing the user will obviously do end to end (these become the architecture's core use cases), plus one where two parties share work, where the product is shared or concurrent.
- **Cross-cutting rules** that the verbs raise, such as undo, go to `DOMAIN.md` under "Invariants (all contexts)".

Write only what the user confirms.

## 5. Close

When the frontier is empty:

1. Park anything still open that isn't a domain question as a GitHub issue (a Task, or a Feature for a new capability) labelled `needs-decision`, with its `area:` label when the capability is known, then delete `docs/domain/OPEN-QUESTIONS.md`.
2. Re-read all of `docs/domain/` and check:
   - no implementation details, no history, no plans
   - no invariant that is only sometimes true
   - every term used in a definition, invariant, or workflow is defined, including meta-terms like "Command"
   - **early definitions still agree with later invariants** (a definition written in round 1 may contradict an invariant from round 5)
   - **every workflow step is possible under the invariants and commands** (e.g. no step that syncs something already detached)
   Fix what you find and say what you fixed.
3. Check `docs/PRODUCT.md`: it exists, states what the product is for and who uses it, and lists three to seven principles (plus a Baseline section if a baseline was named). Ask the user for anything missing.
4. Give the user a short summary: contexts with counts of terms, invariants, commands, events, and workflows; what was deliberately left out and where it went; and any weak spot you noticed but couldn't settle.
