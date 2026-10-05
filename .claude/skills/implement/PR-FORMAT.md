# Pull request format

Remove every section that would be empty or say nothing beyond the diff. Never claim a check or validation that wasn't performed.

```md
## Summary

Why this change is needed and what outcome it gives the user.

## Changes

- Meaningful changes: behaviour and decisions, not files.

## Validation

- Automated: the gates that ran and passed.
- Manual: the manual check the user performed, and what they confirmed.

## Related work

- `Closes #<n>` for the change's issue; otherwise only real references, such as a related pull request or issue. Never a transient file.
```
