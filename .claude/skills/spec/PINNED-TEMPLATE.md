# Pinned spec template

What the system does now. Written to `docs/specs/<capability>/SPEC.md` when a change lands; edited in place after that. Omit sections that would be empty.

```md
# {Capability title}

**Commands**: {Command}, {Command}, {Command}  ← or `none` when the capability owns no Command

## Purpose

Why this capability exists, from the user's perspective. Two or three sentences.

## User Stories

1. As a {role}, I can {behaviour}, so that {benefit}.
2. …

Present tense. Only behaviour the system has today.

## Rules

**{Short rule name}**: {one sentence a test could check}. Follows from: {invariant name}.

## Implementation Decisions

- The modules involved and their interfaces, as built.
- Decisions a future reader needs, each with a `_Why_:` line only where it would otherwise surprise.

No file paths or code.

## Test seams

- **{Short rule name}**: {test reference}, {test reference}

Every Rule appears here with at least one test, each named in the form the project documents in `docs/architecture/ARCHITECTURE.md` (Technology).

## Not supported

Lasting non-goals of this capability: things it deliberately does not do.

## Notes
```
