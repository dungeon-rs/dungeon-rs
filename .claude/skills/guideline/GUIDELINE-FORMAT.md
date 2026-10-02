# Guideline format

```
docs/guidelines/
├── INDEX.md
└── <building-block>.md
```

## INDEX.md

```md
# Guidelines

- [Module wiring](./module-wiring.md): use when adding a module to the application.
- [Undoable Command](./undoable-command.md): use when adding a Command a user can undo.
```

## <building-block>.md

````md
# {Building block}

**Use when**: {situation}. **Not when**: {situation that looks similar but isn't}.
**Exemplar**: `{path/to/the/clearest/instance}`

## Rules

- {3–7 rules no tool can check. Name the lint or check that covers anything else.}

## Example

```{language}
{minimal, complete example, trimmed from the exemplar}
```

## Pitfalls

- {optional: mistakes this building block invites}
````

The example is an excerpt of the exemplar, so it stays true to code that runs: the `guideline-examples` check of `just workspace` fails when any of its lines, indentation aside, is not in the exemplar in the same order.
