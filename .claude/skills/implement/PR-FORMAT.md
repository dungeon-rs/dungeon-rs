# Pull request format

Remove every section that would be empty or say nothing beyond the diff. Never claim a check or validation that wasn't performed. The body is filled when the draft opens and kept true as the work goes; progress and decisions are comments, not edits to the body.

```md
## Summary

Why this change is needed and what outcome it gives the user.

## Changes

- Meaningful changes: behaviour and decisions, not files.

## Validation

- Automated: the gates that ran and passed.
- Manual: the checklist for the author's manual check, and what they confirmed, or that they declined.

## Related work

- `Closes #<n>` for the change's issue; otherwise only real references, such as a related pull request or issue. Never a transient file.
```
