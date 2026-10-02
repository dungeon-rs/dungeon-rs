# Asset Library

The Assets available to an author, where they live, and how a Project refers to them so that it keeps working on any device.

## Language

**Asset**:
A reusable resource an author uses to build maps. Every Asset has an Asset Kind.
_Avoid_: resource, file

**Asset Kind**:
What an Asset is: an image, Material, Shader, Brush Preset, Prefab, or font. Plugins can add kinds. A tileset is an image that an Indexing Rule marks as one, not a kind of its own.

**Asset Folder**:
A folder of Assets that is used as-is, in place. Assets from vendors and the author's own Assets are both simply Asset Folders.
_Avoid_: asset source, custom assets, pack (when meaning a folder)

**Manifest**:
The editor's description of an Asset Folder: its Canonical Name, version, and which of its Assets were renamed. The editor creates it when the folder is added; it is never needed from the vendor.

**Canonical Name**:
The name an Asset Folder is known by in Projects, such as `Forgotten-Adventures`. Chosen when the folder is added; the author keeps it the same on every device.

**Asset Pack**:
An Asset Folder packaged for distribution; it carries content only. Installing an Asset Pack unpacks it into an Asset Folder.
_Avoid_: bundle, archive

**Embedded Asset**:
An Asset stored inside a Project instead of in an Asset Folder, for one-off Assets that belong to that Project alone.

**Bundled File**:
A file the editor ships and needs in order to run, such as its own fonts and shaders. Never an Asset.
_Avoid_: resource, bundled resource

**Indexing Rule**:
A rule that decides which files in an Asset Folder are Assets, and of what Asset Kind.

**Asset Reference**:
What a Project records about an Asset it uses: its name, the Canonical Name of its Asset Folder, every place in that folder it is known to sit, and enough about its content to recognise it elsewhere. It resolves if any of those places holds the Asset.

**Missing Asset**:
An Asset Reference that cannot be resolved on the current device.

**Relink**:
The author's confirmation that a different Asset is the one an Asset Reference means. It adds a place to the Asset Reference.

**Brush Preset**:
An Asset that stores a Brush's settings: size, texture, strength, and any further settings a Brush defines.

**Material**:
The look of a surface: the Shader that renders it and the parameter values it uses. Industry-standard meaning.

**Shader**:
An Asset that defines how a surface is rendered and which parameters a Material can set. Industry-standard meaning.

**Prefab**:
An Asset that holds a reusable arrangement of Elements, placed as Prefab Instances.

## Invariants

**A Missing Asset is always explainable**: an Asset Reference alone is enough to tell the author, in plain terms, which Asset is missing, which Asset Folder it came from, and what uses it.

**A Relink never removes what an Asset Reference already knows.**
_Why_: collaborators with different versions of an Asset Folder would otherwise undo each other's Relinks.

**Canonical Names are unique on a device**: no two Asset Folders on the same device share a Canonical Name.

**An Embedded Asset is stored once per Project**, however many Elements use it.

**Asset Packs are never used directly**: an Asset Pack's Assets are only available once it is installed as an Asset Folder.
_Why_: unpacking on every load is what makes very large libraries slow to open.

## Commands

**Add Asset Folder**: make a folder's Assets available, creating its Manifest and asking for its Canonical Name.
**Remove Asset Folder**: stop using a folder's Assets on this device.
**Rename Canonical Name**: change the name an Asset Folder is known by. Projects using the old name see its Assets as Missing until the names match again.
**Install Asset Pack**: unpack an Asset Pack into a new Asset Folder.
**Embed Asset**: store an Asset inside the current Project.
**Relink**: add a place to an Asset Reference.
**Update Prefab**: change a Prefab, choosing whether its linked Prefab Instances follow, are detached, or stay linked and out of date (the default).

## Events

**Asset Folder Changed**: an Asset Folder became available or stopped being available on this device, or Assets in it were added, removed, or changed, for example by a vendor update. Listened to by: Authoring, which re-resolves the Asset References pointing into it.

**Prefab Updated**: a Prefab changed. Listened to by: Authoring, which updates, detaches, or marks as out of date the linked Prefab Instances, as chosen in Update Prefab.

## Workflows

### Use a new vendor release
1. The vendor's files land in a new or existing Asset Folder; nothing editor-specific is needed from the vendor.
2. Indexing Rules decide which files are Assets and of what Asset Kind.
3. The Assets are available to place.

### Open a Project on another device
1. Each Asset Reference is resolved through the Canonical Name of its Asset Folder and the places it is known to sit.
2. Any that can't be resolved become Missing Assets; their Elements stay in the Project, and the author is told in plain terms what is missing and what it affects.
3. The author may Relink Missing Assets, or leave them; saving keeps them either way.

### Share a Project with a collaborator
1. The collaborator opens the Project; Assets resolve through any place their Asset Reference knows.
2. Missing Assets name their Asset Folder and the version the Project was saved against.
3. A Relink the collaborator makes adds a place, so the Project still resolves for everyone who had it working before.

### Mass-update a Prefab
1. The author edits a Prefab.
2. The author chooses: its linked Prefab Instances follow, are detached, or stay linked and out of date.
3. Out-of-date Prefab Instances can later be synced one at a time.
