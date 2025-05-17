The UI package handles, as the name implies, building the user interface.
This is consumed from the `editor` package, and is currently written in
the wonderful [`egui`](https://github.com/emilk/egui) library.

While Bevy provides it's own [UI library](https://docs.rs/bevy/latest/bevy/ui),
it's currently not mature enough in my opinion compared to `egui` and the like.

This decision may be revised at a later time, with minimal impact on the rest
of the application.