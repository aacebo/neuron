use std::path::{Path, PathBuf};

use crate::resources::{Asset, AssetData, Repository};
use crate::{Error, Result};

pub struct FileSystem {
    root: PathBuf,
}

impl FileSystem {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Repository for FileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.root.join(path).exists()
    }

    fn get(&self, path: &Path) -> Result<Asset> {
        let path = self.resolve(path)?;

        match path.is_dir() {
            true => Ok(Asset::directory(path)),
            false => Ok(Asset::file(path)),
        }
    }

    fn read(&self, path: &Path) -> Result<AssetData> {
        let path = self.resolve(path)?;

        if !path.is_dir() {
            return Ok(AssetData::File(std::fs::read(path).map_err(Error::load)?));
        }

        let mut assets = Vec::new();

        for entry in std::fs::read_dir(path).map_err(Error::load)? {
            let entry = entry.map_err(Error::load)?;
            let path = entry.path();

            assets.push(match path.is_dir() {
                true => Asset::directory(path),
                false => Asset::file(path),
            });
        }

        Ok(AssetData::Directory(assets))
    }

    fn resolve(&self, path: &Path) -> Result<PathBuf> {
        let path = self.root.join(path);

        match path.exists() {
            true => Ok(path),
            false => Err(Error::Load(format!("{} not found", path.display()))),
        }
    }
}
