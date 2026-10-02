//! The asset source thumbnails are read through: `thumb://<folder-key>/<place>`.

use super::pack::{Digest, Record, read_entry};
use crate::LibraryTable;
use crate::source::key_and_parts;
use bevy_app::App;
use bevy_asset::AssetApp;
use bevy_asset::io::{
    AssetReader, AssetReaderError, AssetReaderFuture, AssetSourceBuilder, PathStream, Reader,
    VecReader,
};
use bevy_ecs::resource::Resource;
use bevy_tasks::ConditionalSendFuture;
use drs_model::{FolderKey, THUMBNAIL_SOURCE};
use std::collections::HashMap;
use std::fs::File;
use std::future::ready;
use std::path::Path;
use std::sync::{Arc, PoisonError, RwLock};

/// Which thumbnail serves each Asset at its current byte size and modification time, shared
/// with the `thumb://` reader and changed at runtime as thumbnails are looked up and generated.
#[derive(Resource, Clone, Default)]
pub struct ThumbnailTable {
    /// The pack and the records served.
    served: Arc<RwLock<Served>>,
}

/// What the `thumb://` reader serves.
#[derive(Default)]
struct Served {
    /// The pack, open for positional reads, once the thumbnail cache is open.
    pack: Option<Arc<File>>,
    /// The record serving each Asset, by the digest of its folder key and place.
    records: HashMap<Digest, Record>,
}

/// The digest of an Asset's folder key and place, which the table is keyed by.
pub(crate) fn place_digest(folder: &str, place: &str) -> Digest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(folder.as_bytes());
    hasher.update(&[0]);
    hasher.update(place.as_bytes());
    let mut digest = [0; 16];
    digest.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    digest
}

impl ThumbnailTable {
    /// Serves thumbnails out of `pack` from now on.
    pub(crate) fn attach(&self, pack: Arc<File>) {
        self.served
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .pack = Some(pack);
    }

    /// Serves the Asset at `place` in the folder with `key` from `record`, or nothing when
    /// `record` is `None` or broken.
    pub(crate) fn serve(&self, key: &FolderKey, place: &str, record: Option<Record>) {
        let digest = place_digest(key.as_str(), place);
        let mut served = self.served.write().unwrap_or_else(PoisonError::into_inner);
        match record.filter(|record| !record.is_broken()) {
            Some(record) => {
                served.records.insert(digest, record);
            }
            None => {
                served.records.remove(&digest);
            }
        }
    }

    /// The encoded thumbnail served for the Asset at `place` in the folder with `key`.
    ///
    /// # Errors
    ///
    /// [`AssetReaderError::NotFound`] when no thumbnail is served for it, or
    /// [`AssetReaderError::Io`] when the pack cannot be read.
    fn bytes(&self, key: &str, place: &str, path: &Path) -> Result<Vec<u8>, AssetReaderError> {
        let not_found = || AssetReaderError::NotFound(path.to_path_buf());
        let (pack, record) = {
            let served = self.served.read().unwrap_or_else(PoisonError::into_inner);
            let record = served
                .records
                .get(&place_digest(key, place))
                .copied()
                .ok_or_else(not_found)?;
            (served.pack.clone().ok_or_else(not_found)?, record)
        };
        read_entry(&pack, record).map_err(|error| AssetReaderError::Io(Arc::new(error)))
    }
}

/// Reads thumbnails out of the pack.
struct ThumbnailReader {
    /// The folders `lib://` serves, so that a path it refuses is refused here too.
    library: LibraryTable,
    /// The thumbnails served.
    table: ThumbnailTable,
}

impl ThumbnailReader {
    /// The encoded thumbnail of the Asset an asset path names.
    ///
    /// # Errors
    ///
    /// [`AssetReaderError::NotFound`] when `lib://` would refuse the path or no thumbnail is
    /// served for it, or [`AssetReaderError::Io`] when the pack cannot be read.
    fn read_thumbnail(&self, path: &Path) -> Result<VecReader, AssetReaderError> {
        self.library.resolve(path)?;
        let not_found = || AssetReaderError::NotFound(path.to_path_buf());
        let (key, parts) = key_and_parts(path).ok_or_else(not_found)?;
        let parts: Option<Vec<&str>> = parts.into_iter().map(std::ffi::OsStr::to_str).collect();
        let place = parts.ok_or_else(not_found)?.join("/");
        Ok(VecReader::new(self.table.bytes(key, &place, path)?))
    }
}

// The reads are synchronous: one thumbnail is a few kilobytes read at a known offset, so each
// future is ready at once.
impl AssetReader for ThumbnailReader {
    fn read<'a>(&'a self, path: &'a Path) -> impl AssetReaderFuture<Value: Reader + 'a> {
        ready(self.read_thumbnail(path))
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
        ready(self.library.resolve(path).map(|_| false))
    }
}

/// Registers the `thumb://` asset source and the [`ThumbnailTable`] its reader serves from.
///
/// Asset sources freeze when Bevy's `AssetPlugin` builds, so the Host calls this before adding
/// it, after [`register_library_source`](crate::register_library_source), whose table the reader
/// shares. The table is returned and also inserted as a resource; a table already in the `World`
/// is reused.
pub fn register_thumbnail_source(app: &mut App) -> ThumbnailTable {
    let table = app
        .world()
        .get_resource::<ThumbnailTable>()
        .cloned()
        .unwrap_or_default();
    let library = app
        .world()
        .get_resource::<LibraryTable>()
        .cloned()
        .unwrap_or_default();
    app.insert_resource(table.clone())
        .insert_resource(library.clone());
    let reader_table = table.clone();
    app.register_asset_source(
        THUMBNAIL_SOURCE,
        AssetSourceBuilder::new(move || {
            Box::new(ThumbnailReader {
                library: library.clone(),
                table: reader_table.clone(),
            })
        }),
    );
    table
}
