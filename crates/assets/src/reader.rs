//! Implements a custom `AssetReader` to integrate the crate into Bevy's asset system.

use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader};
use std::path::Path;

pub struct DrsAssetReader;

impl AssetReader for DrsAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        // ...
        let val: Box<dyn Reader> = unimplemented!();
        Ok(val)
    }
    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let val: Box<dyn Reader> = unimplemented!();
        Ok(val)
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        unimplemented!()
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        unimplemented!()
    }

    async fn read_meta_bytes<'a>(&'a self, path: &'a Path) -> Result<Vec<u8>, AssetReaderError> {
        unimplemented!()
    }
}
