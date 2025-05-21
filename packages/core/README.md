The core package is where most of the logic lives.
It defines the ECS hierarchy, which provides the building blocks for the projects,
it handles persistence, export and data structures.

## Key Capabilities

| Area                   | Highlights                                                                                                                         |
|------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| Hierarchical ECS Model | `Project`, `Level`, `Layer`, and `Texture`—simple, self-describing components that model a dungeon map.                            |                                       |
| Persistence            | `SaveFile` captures the complete map state (including semantic version, canvas size, and assets) and re-instantiates it on demand. |
| Minimal & Stable Deps  | Only trusted, widely-used crates (`bevy`, `serde`, `semver`, `image`, …).                                                          |

## Conceptual Model

A `DungeonRS` map is expressed as a strict three-level hierarchy:

* **Project** – The root entity; defines the overall canvas (`Rect`) and project metadata.  
* **Level** – Represents a discrete floor or depth within the project; multiple levels are allowed.  
* **Layer** – Z-sorted collection of renderables; typical use is background, props, foreground, etc.  
