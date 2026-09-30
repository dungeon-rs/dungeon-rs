# Change spec template

Behaviour not implemented yet. Written to `docs/changes/<slug>.md`, deleted when it lands. Omit sections that would be empty.

```md
# {Change title}

**Capabilities**:
- {capability}: {Command}, {Command}
- {capability}: {Command}

## Problem Statement

The problem the user faces, from the user's perspective.

## Solution

The solution, from the user's perspective.

## User Stories

1. As a {role}, I want {behaviour}, so that {benefit}.
2. …

A long list. One behaviour per story, in domain terms. Cover edge cases and the two-parties-different-setups case where it applies.

## Rules

**{Short rule name}**: {one sentence a test could check}. Follows from: {invariant name}.

## Changes to existing behaviour

- {capability} — **{existing rule name}**: modified to {new statement} / removed, because {reason}.

## Implementation Decisions

- The modules built or changed, and their interfaces, in terms of `docs/architecture/`.
- Technical clarifications and trade-offs decided with the user.

No file paths or code.

## Testing

- The agreed seams: the highest point that exercises the behaviour, as few as possible.
- Which Rules each seam covers.

## Out of Scope

What this change deliberately leaves for later.

## Further Notes
```
