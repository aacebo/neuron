use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::bert;

use super::config::Config;
use crate::models::Forward;
use crate::{Error, Result};

pub struct Bert {
    inner: bert::BertModel,
}

impl Bert {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self> {
        Ok(Self {
            inner: bert::BertModel::load(vars, &bert::Config::from(config)).map_err(Error::load)?,
        })
    }

    /// `mask` is the keep-mask (1 = real token); `BertModel` widens it internally.
    pub fn forward(&self, ids: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let types = ids.zeros_like().map_err(Error::inference)?;
        self.inner.forward(ids, &types, Some(mask)).map_err(Error::inference)
    }
}

impl Forward for Bert {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, mask): Self::Input) -> Result<Self::Output> {
        self.forward(&ids, &mask)
    }
}
