use candle_core::{DType, Device};
use tokenizers::Tokenizer;

use super::capability::{Classify, Context, Embed, Generate, TokenClassify};
use super::{Architecture, LocalModel, ModelRef, RemoteModel, bart, bert, distilbert};
use crate::{Error, Result};

/// A model with its weights loaded (or, for a remote model, its endpoint bound). The variant
/// determines which capabilities exist: the `None` arms below are the empty cells of the matrix.
pub enum Model {
    Embedder(bert::Embedder),
    TokenClassifier(bert::TokenClassifier),
    SequenceClassifier(distilbert::SequenceClassifier),
    Summarizer(bart::Summarizer),
    Remote(RemoteModel),
}

/// A loaded model plus the tokenizer and device it needs to see text. Lends a [`Context`] per
/// call, so one set of weights serves every capability the model has.
pub struct Loaded {
    model: Model,
    tokenizer: Option<Tokenizer>,
    device: Device,
    name: String,
}

impl Loaded {
    pub fn new(model: &ModelRef, api_key: &Option<String>, device: Device, dtype: DType) -> Result<Self> {
        let name = model.to_string();

        match model {
            ModelRef::Remote(remote) => Ok(Self {
                model: Model::Remote(remote.clone().api_key(api_key.clone())),
                tokenizer: None,
                device,
                name,
            }),
            ModelRef::Local(local) => Self::local(local, device, dtype, name),
        }
    }

    fn local(model: &LocalModel, device: Device, dtype: DType, name: String) -> Result<Self> {
        let repo = model.loader(device, dtype)?;
        let device = repo.device().clone();
        let tokenizer = repo.tokenizer()?;

        // `config.json` names the architecture; the architecture decides which capabilities the
        // weights can serve.
        let architecture: Probe = repo.config()?;

        let model = match architecture.architecture {
            Architecture::Bart => {
                let config: bart::Config = repo.config()?;
                Model::Summarizer(bart::Summarizer::new(&config, repo.vars()?)?)
            }
            Architecture::DistilBert => {
                let config: distilbert::Config = repo.config()?;
                Model::SequenceClassifier(distilbert::SequenceClassifier::new(repo.vars()?, &config)?)
            }
            // A BERT checkpoint is an embedder or a token classifier depending on whether it
            // carries a label map -- a classification head is exactly what `id2label` announces.
            Architecture::Bert | Architecture::Unknown => {
                let config: bert::Config = repo.config()?;

                match config.has_labels() {
                    true => Model::TokenClassifier(bert::TokenClassifier::new(repo.vars()?, &config)?),
                    false => Model::Embedder(bert::Embedder::new(repo.vars()?, &config)?),
                }
            }
            other => {
                return Err(Error::Load(format!("{name} has unsupported architecture `{other}`")));
            }
        };

        Ok(Self {
            model,
            tokenizer: Some(tokenizer),
            device,
            name,
        })
    }

    /// Infallible: a hosted model legitimately has no tokenizer, and its capability impls never
    /// ask for one.
    pub fn context(&self) -> Context<'_> {
        Context::new(self.tokenizer.as_ref(), &self.device, &self.name)
    }

    pub fn as_embed(&self) -> Option<&dyn Embed> {
        match &self.model {
            Model::Embedder(model) => Some(model),
            Model::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_classify(&self) -> Option<&dyn Classify> {
        match &self.model {
            Model::SequenceClassifier(model) => Some(model),
            Model::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_token_classify(&self) -> Option<&dyn TokenClassify> {
        match &self.model {
            Model::TokenClassifier(model) => Some(model),
            Model::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_generate(&self) -> Option<&dyn Generate> {
        match &self.model {
            Model::Summarizer(model) => Some(model),
            Model::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn cannot(&self, capability: &str) -> Error {
        Error::Inference(format!("{} cannot {capability}", self.name))
    }
}

/// Reads just the architecture out of `config.json`, before committing to a concrete config type.
#[derive(serde::Deserialize)]
struct Probe {
    #[serde(default, rename = "model_type")]
    architecture: Architecture,
}
