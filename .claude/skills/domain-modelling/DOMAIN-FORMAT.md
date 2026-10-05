# Domain format

```
docs/
├── PRODUCT.md                ← vision and principles (not domain facts)
├── domain/
│   ├── DOMAIN.md             ← purpose, context map, relationships, cross-cutting invariants
│   ├── <context>.md          ← one file per bounded context
│   └── OPEN-QUESTIONS.md     ← working file; exists only while questions are open
```

A small domain starts as a single `DOMAIN.md` with a Language section. Split into context files when it holds roughly 25 terms or no longer fits a screen or two.

## PRODUCT.md

```md
# Product

{Two or three sentences: what the product is, for whom, and what makes it different.}

## Principles

- **{Short name}.** {One or two sentences. A lasting value every spec must respect, not a feature.}
```

## Baseline

Optional. Only when an existing product serves as a functional baseline.

```md
## Baseline

{Product} is the functional baseline; this product differs in {how}.

Pain points that matter:
- {short pain point, with its source}
```

The architecture quotes these as the sources of Needs.

Three to seven principles. Quality requirements that are lasting values ("fast at any catalogue size") are principles; concrete targets ("opens in under 2 s") are spec concerns.

## DOMAIN.md

```md
# {Product name}

{Two or three sentences: what the product is for and who uses it.}

## Contexts

- [Ordering](./ordering.md): composing Orders out of Line Items
- [Catalogue](./catalogue.md): the Products on offer and how an Order refers to them

## Relationships

- **Ordering → Catalogue**: Line Items refer to Products only through Product References; the Catalogue resolves them.

## Invariants (all contexts)

**Every Command can be undone**: every Command listed in any context, without exception.
```

## <context>.md

```md
# {Context name}

{One or two sentences: what this context is responsible for and why it is separate.}

## Language

**Line Item**:
One Product and a quantity within an Order.
_Avoid_: row, entry, position

**Transaction**:
An all-or-nothing change to stored data. Industry-standard meaning.

## Invariants

**Nothing is fixed at creation**: every property of a Line Item stays editable after it is added.
_Why_: {only when the rule would surprise a future reader}

## Commands

**Add Line Item**: add a Line Item to an Order.

## Events

**Product Discontinued**: a Product in the Catalogue was withdrawn. Listened to by: Ordering, which re-resolves the Product References pointing to it.

## Workflows

### Reopen an Order on another device
1. …
```

## OPEN-QUESTIONS.md

```md
# Open questions

Working file for the domain interview. Removed when empty.

## Round {n} (asked)
- **{title}** `[gap|ambiguity|contradiction]`: {one line}

## Later
- **{title}**: {one line}
```

## Rules

- **Be opinionated.** When several words exist for one concept, pick one and list the rest under `_Avoid_`.
- **Define what a thing IS, not what it does.** One or two sentences.
- **Industry-standard terms** get a one-line definition marked "Industry-standard meaning." Don't redefine them, and don't let another concept use their name.
- **Only project-specific terms.** General programming concepts never belong.
- **Invariants are always true**, in every state, after every command. Something that is only true sometimes is a rule of a command, not an invariant. Give each one a short bold name; specs and Needs cite it by that name.
- **Commands are user intentions** that change the domain, named as verbs in the imperative. Include only those that matter to the domain, not every UI gesture; property changes can share one "Edit X" command.
- **Events only when another context reacts to them**, and say how it reacts.
- **Workflows are short.** Steps in domain language; no screens, buttons, or shortcuts.
- **Present tense, current state.** No history, no "we used to", no plans.
- **Omit empty sections.**
