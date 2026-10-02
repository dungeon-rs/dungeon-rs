//! The one dynamic asset source every Asset Folder is read through: `lib://<folder-key>/<place>`.

use bevy_app::App;
use bevy_asset::AssetApp;
use bevy_asset::io::{
    AssetReader, AssetReaderError, AssetReaderFuture, AssetSourceBuilder, PathStream, Reader,
    VecReader,
};
use bevy_ecs::resource::Resource;
use bevy_tasks::ConditionalSendFuture;
use drs_model::FolderKey;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::future::ready;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock};

/// The name of the asset source: paths read `lib://<folder-key>/<place>`.
pub const LIBRARY_SOURCE: &str = "lib";

/// The folders the `lib://` source can read, by key. Shared with the reader and changed at
/// runtime as folders are added and forgotten.
#[derive(Resource, Clone, Default)]
pub struct LibraryTable {
    /// Folder key to folder path.
    folders: Arc<RwLock<BTreeMap<String, PathBuf>>>,
}

impl LibraryTable {
    /// Makes the folder at `path` readable under `key`.
    pub(crate) fn insert(&self, key: &FolderKey, path: PathBuf) {
        self.folders
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.as_str().to_owned(), path);
    }

    /// Stops reading the folder under `key`.
    pub(crate) fn remove(&self, key: &FolderKey) {
        self.folders
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(key.as_str());
    }

    /// The file an asset path (the part after `lib://`) names, refusing anything that is not a
    /// plain relative path below a registered folder.
    ///
    /// # Errors
    ///
    /// [`AssetReaderError::NotFound`] when the key is not registered or the path is not plain.
    pub(crate) fn resolve(&self, path: &Path) -> Result<PathBuf, AssetReaderError> {
        let not_found = || AssetReaderError::NotFound(path.to_path_buf());
        let (key, parts) = key_and_parts(path).ok_or_else(not_found)?;
        let mut file = self
            .folders
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(key)
            .cloned()
            .ok_or_else(not_found)?;
        file.extend(parts);
        Ok(file)
    }
}

/// The folder key an asset path (the part after the source's `://`) starts with and the parts
/// of the place that follow it, or `None` when the path is not a key followed by a plain
/// relative path: one that is empty, absolute, or climbs with `..` is refused.
pub(crate) fn key_and_parts(path: &Path) -> Option<(&str, Vec<&OsStr>)> {
    let mut components = path.components();
    let Some(Component::Normal(key)) = components.next() else {
        return None;
    };
    let key = key.to_str()?;
    let mut parts = Vec::new();
    for component in components {
        match component {
            Component::Normal(part) => parts.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!parts.is_empty()).then_some((key, parts))
}

/// Reads Assets out of the registered folders.
struct LibraryReader {
    /// The folders to read from.
    table: LibraryTable,
}

impl LibraryReader {
    /// Reads the whole file an asset path names.
    ///
    /// # Errors
    ///
    /// [`AssetReaderError::NotFound`] when the path resolves to no file, or
    /// [`AssetReaderError::Io`] when the file cannot be read.
    fn read_file(&self, path: &Path) -> Result<VecReader, AssetReaderError> {
        let file = self.table.resolve(path)?;
        let bytes = std::fs::read(&file).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AssetReaderError::NotFound(path.to_path_buf())
            } else {
                AssetReaderError::Io(Arc::new(error))
            }
        })?;
        Ok(VecReader::new(bytes))
    }
}

// The reads are synchronous: a folder on disk is read in one go, as Bevy's own file reader does
// for small files, so each future is ready at once.
impl AssetReader for LibraryReader {
    fn read<'a>(&'a self, path: &'a Path) -> impl AssetReaderFuture<Value: Reader + 'a> {
        ready(self.read_file(path))
    }

    fn read_meta<'a>(&'a self, path: &'a Path) -> impl AssetReaderFuture<Value: Reader + 'a> {
        ready(Err::<VecReader, _>(AssetReaderError::NotFound(
            path.to_path_buf(),
        )))
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<PathStream>, AssetReaderError>> {
        ready(Err(AssetReaderError::NotFound(path.to_path_buf())))
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<bool, AssetReaderError>> {
        ready(self.table.resolve(path).map(|file| file.is_dir()))
    }
}

/// Registers the `lib://` asset source and the [`LibraryTable`] its reader looks folders up in.
///
/// Asset sources freeze when Bevy's `AssetPlugin` builds, so the Host calls this before adding
/// it. The table is returned and also inserted as a resource; a table already in the `World` is
/// reused.
pub fn register_library_source(app: &mut App) -> LibraryTable {
    let table = app
        .world()
        .get_resource::<LibraryTable>()
        .cloned()
        .unwrap_or_default();
    app.insert_resource(table.clone());
    let reader_table = table.clone();
    app.register_asset_source(
        LIBRARY_SOURCE,
        AssetSourceBuilder::new(move || {
            Box::new(LibraryReader {
                table: reader_table.clone(),
            })
        }),
    );
    table
}
