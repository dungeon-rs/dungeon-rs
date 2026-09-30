# Extension

Plugins that add behavior to the editor.

## Language

**Plugin**:
An installable extension that adds behavior to the editor: Indexing Rules, Asset Kinds, Brushes, and Element kinds. A Plugin may also include Assets, which it provides the way an Asset Folder does. Every Plugin has a name and a version; an Element kind a Plugin adds is known by the Plugin's name together with the kind's own name.
_Avoid_: mod, script

**Missing Element Kind**:
The kind of an Element whose Plugin is not installed on the current device.

## Invariants

**A Missing Element Kind is always explainable**: the Project alone is enough to tell the Author, in plain terms, which Plugin is missing, which version the Project was saved with, and what uses it.
