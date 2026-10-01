# drs-project-access

The `ResourceAccess` that reads and writes Project files.

Its contract: [`read_project`](crate::read_project) and
[`write_project`](crate::write_project). A Project file is one JSON document in
the shape the model's `ProjectFile` describes, pretty-printed in a fixed key
order. Reading checks the format version first, then asks the model's
serialisation registry whether every known component was written at a version
this editor reads, and keeps the envelopes the registry does not know as they
are; a newer format or component is refused by name and version, never opened
with parts missing. Writing goes through a temporary file beside the target that
is renamed over it once complete, so a failed write leaves the previous file
untouched and never a partial one.

Nothing here names a component: which components exist, and how each is read
and written, is the registry's business.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
