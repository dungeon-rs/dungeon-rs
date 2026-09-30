# Roadmap format

```md
# Roadmap

## {Milestone: an outcome a user could state}

- **{Change title}**: {one-line outcome}. Commands: {Command}, {Command}.
- **[{Change title}](changes/{slug}.md)**: {outcome}. Commands: {Command}.
- **{Question title}**: Answer: {the question}. Commands: none.

## {Next milestone}

- …

## {Later milestone}

{One-line outcome. No change lines yet.} Commands: {Command}, {Command}.
```

## Rules

- Lines are in build order within a milestone; milestones are in build order.
- `Commands:` lists the domain Commands the change will claim once its spec exists, exactly as the domain names them; `none` when it covers no new Command (plumbing, questions, or behaviour driven by Commands another capability owns). A sketched milestone lists the Commands it will cover after its outcome, so the review's coverage checklist sees them as planned.
- A line links to its change spec once `/spec` has written one.
- Landed changes and finished milestones are deleted, not marked done.
