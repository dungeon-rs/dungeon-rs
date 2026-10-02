# drs-project-access

The `ResourceAccess` that reads and writes Project files.

Its contract: [`read_project`](crate::read_project) and
[`write_project`](crate::write_project), between a file and the model's
`ProjectSnapshot`: the Project's envelopes, its Levels and Layers in order, and
its Elements by identity, in no file's shape. The shape of the file, and its
version ([`FORMAT_VERSION`](crate::FORMAT_VERSION)), live here and nowhere else: a
Project file is one JSON document of the format version, the Project's
envelopes, the Levels with their Layers, and the Elements, pretty-printed in a
fixed key order. Reading checks the format version first, then asks the model's
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
