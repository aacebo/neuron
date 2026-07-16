use std::path::{Path, PathBuf};

use crate::resources::{Asset, AssetData, Repository, Uri, cache};
use crate::{Error, Result};

pub struct Http {
    base: Uri,
}

impl Http {
    pub fn new(base: Uri) -> Self {
        Self { base }
    }

    fn cached(&self, file: &str) -> PathBuf {
        cache::dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("urls")
            .join(slug(&self.base.to_string()))
            .join(file)
    }
}

impl Repository for Http {
    fn exists(&self, path: &Path) -> bool {
        self.resolve(path).is_ok()
    }

    fn get(&self, path: &Path) -> Result<Asset> {
        Ok(Asset::file(self.resolve(path)?))
    }

    fn read(&self, path: &Path) -> Result<AssetData> {
        let path = self.resolve(path)?;
        Ok(AssetData::File(std::fs::read(path).map_err(Error::load)?))
    }

    fn resolve(&self, path: &Path) -> Result<PathBuf> {
        let file = path.to_str().ok_or_else(|| Error::Load(format!("{path:?} is not utf-8")))?;
        let cached = self.cached(file);

        if cached.exists() {
            return Ok(cached);
        }

        let url = self.base.join(file)?.to_string();
        let response = reqwest::blocking::get(&url).map_err(Error::network)?;

        if !response.status().is_success() {
            return Err(Error::Network(format!("{url} returned {}", response.status())));
        }

        let bytes = response.bytes().map_err(Error::network)?;

        if let Some(dir) = cached.parent() {
            std::fs::create_dir_all(dir).map_err(Error::load)?;
        }

        std::fs::write(&cached, &bytes).map_err(Error::load)?;

        Ok(cached)
    }
}

fn slug(base: &str) -> String {
    base.chars().map(|ch| if ch.is_alphanumeric() { ch } else { '-' }).collect()
}
