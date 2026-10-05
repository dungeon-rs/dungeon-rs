# Plan format

The plan lives on GitHub, read and written with `gh`. Nothing in the repository lists planned work.

## Milestone

An outcome a user could state, as the title ("I can build and export a simple dungeon"); one line in the description. Milestones are in build order, which is the order they were created. A sketched later milestone is a title and a description, with at most one sketch issue per thing it names.

## Change

A **Feature** issue in its milestone.

```md
{One-line outcome.}

**Commands**: {Command}, {Command}.

Change spec: {URL of docs/changes/{slug}.md}
```

- Title: the change's title, as the domain would say it.
- `Commands` lists the domain Commands the change will claim once its spec exists, exactly as the domain names them; `none` when it covers no new Command (plumbing, questions, or behaviour driven by Commands another capability owns). The review's coverage checklist reads these lines.
- The `Change spec:` line is added by `/spec` when it writes the change spec, and `spec:new` is removed then.
- Labels: `area:<capability>` once the capability is known, and `spec:new` until a change spec exists.

## Question

A **Task** issue titled "Answer: {the question}", in the milestone, placed before the change that depends on it with a blocked-by relationship. It ends in a conclusion folded into the domain, the change spec, or the architecture (with a `_Why_:` line), and is then closed with that conclusion as its closing comment.

## Labels

| Label | Meaning |
|---|---|
| `area:<capability>` | the capability the issue belongs to |
| `spec:none` | brings the code in line with a Rule already pinned |
| `spec:change` | alters a pinned Rule, so it goes through `/spec` first |
| `spec:new` | needs a Rule or capability that does not exist yet |
| `needs-decision` | waiting on the author |
| `blocked` | waiting on another change |
| `polish` | correct but feels wrong in use |
| `test-gap` | a Rule or behaviour without enough tests |
| `tech-debt` | structure or enforcement, no behaviour change |

## Rules

- Build order is milestone order, then dependencies (blocked-by), then issue number. To put an issue ahead of another, make the other blocked by it.
- Issues outside any milestone are the backlog: adjustments, parked decisions, and ideas. An issue joins a milestone when it is planned.
- An issue being worked is assigned to the author's account and has a draft pull request that closes it.
- Landed changes are closed by their pull request, and a milestone is closed when its last issue is. Nothing is edited out or marked done in the body.
