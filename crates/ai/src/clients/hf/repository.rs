use std::path::{Path, PathBuf};

use hf_hub::api::sync::{ApiBuilder, ApiRepo};

use crate::resources::{Asset, AssetData, ModelId, Repository, cache};
use crate::{Error, Result};

pub struct HuggingFace {
    repo: Box<ApiRepo>,
}

impl HuggingFace {
    pub fn new(id: &ModelId) -> Result<Self> {
        let mut builder = ApiBuilder::new().with_token(std::env::var("HF_TOKEN").ok());

        if let Some(dir) = cache::dir() {
            builder = builder.with_cache_dir(dir);
        }

        let api = builder.build().map_err(Error::load)?;

        Ok(Self {
            repo: Box::new(api.model(id.to_string())),
        })
    }
}

impl Repository for HuggingFace {
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
        self.repo.get(file).map_err(Error::load)
    }
}
