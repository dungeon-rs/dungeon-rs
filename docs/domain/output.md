# Output

Turning a Level into something usable outside the editor.

## Language

**Export**:
An image of one Level, covering exactly the Project's Bounds, at a resolution the author chooses.

## Invariants

**Only what is visible is exported**: hidden Layers, Elements in hidden Layer Groups, and Trace Images never appear in an Export.

## Commands

**Export Level**: produce an Export of one Level at a chosen resolution.

## Workflows

### Export a Level
1. The Author chooses a Level and a resolution.
2. Everything visible inside the Bounds is rendered.
3. The result is an image.
