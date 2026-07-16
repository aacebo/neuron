pub mod bart;
pub mod bert;
pub mod distilbert;

mod capability;
mod loaded;
mod loader;
mod model_id;
mod provider;
mod remote;
mod tokenizer;

use std::str::FromStr;
use std::sync::Arc;

use candle_core::{DType, Device};
pub use capability::{Classify, Context, Embed, GenOpts, Generate, Label, TokenClassify, Word};
pub use loaded::Loaded;
pub use loader::Loader;
pub use model_id::ModelId;
pub use provider::Provider;

use crate::clients::fs::FileSystem;
use crate::clients::hf::HuggingFace;
use crate::clients::http::Http;
use crate::clients::openai::OpenAI;
use crate::resources::{Repository, Resource, Uri};
use crate::{Error, Result};

pub trait Forward: Send + Sync {
    type Input;
    type Output;

    fn forward(&self, input: Self::Input) -> Result<Self::Output>;
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    Bart,
    Bert,
    DistilBert,
    Roberta,
    Deberta,
    Gpt2,
    Llama,
    Mistral,
    T5,
    #[default]
    #[serde(other)]
    Unknown,
}

impl Architecture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bart => "bart",
            Self::Bert => "bert",
            Self::DistilBert => "distilbert",
            Self::Roberta => "roberta",
            Self::Deberta => "deberta",
            Self::Gpt2 => "gpt2",
            Self::Llama => "llama",
            Self::Mistral => "mistral",
            Self::T5 => "t5",
            Self::Unknown => "??",
        }
    }
}

impl FromStr for Architecture {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Ok(match value.to_lowercase().as_str() {
            "bart" => Self::Bart,
            "bert" => Self::Bert,
            "distilbert" => Self::DistilBert,
            "roberta" => Self::Roberta,
            "deberta" => Self::Deberta,
            "gpt2" => Self::Gpt2,
            "llama" => Self::Llama,
            "mistral" => Self::Mistral,
            "t5" => Self::T5,
            _ => Self::Unknown,
        })
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A model whose weights we load and run ourselves. Every variant has weights, so `repository`
/// and `loader` are total -- a model with no weights cannot be built as a `LocalModel`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalModel {
    Hub(ModelId),
    Path(Resource),
}

impl LocalModel {
    pub fn loader(&self, device: Device, dtype: DType) -> Result<Loader> {
        Ok(Loader::new(self.repository()?, device, dtype))
    }

    pub fn repository(&self) -> Result<Arc<dyn Repository>> {
        match self {
            Self::Hub(id) => Ok(Arc::new(HuggingFace::new(id)?)),
            Self::Path(resource) => match &resource.uri {
                Uri::Local(path) => Ok(Arc::new(FileSystem::new(path))),
                Uri::Http(_) => Ok(Arc::new(Http::new(resource.uri.clone()))),
            },
        }
    }
}

impl std::fmt::Display for LocalModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hub(id) => write!(f, "{id}"),
            Self::Path(resource) => write!(f, "{resource}"),
        }
    }
}

/// A model we call over the wire. It has an endpoint, never weights.
#[derive(Debug, Clone)]
pub struct RemoteModel {
    pub provider: Provider,
    pub id: ModelId,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

impl RemoteModel {
    pub fn new(provider: Provider, id: ModelId) -> Self {
        Self {
            provider,
            id,
            base_url: None,
            api_key: None,
        }
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        self.base_url = url;
        self
    }

    pub fn api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn client(&self) -> OpenAI {
        match self.provider {
            Provider::OpenAI => OpenAI::new(self.id.clone(), self.base_url.clone(), self.api_key.clone()),
        }
    }
}

/// Identity is the endpoint, not the credential. `RemoteModel` sits inside the cache
/// [`Key`](crate::pipelines::Key), which fingerprints the api key rather than storing it -- so
/// hashing the raw key here would put it in a long-lived map, and two keys against the same
/// endpoint would wrongly look like different models.
impl PartialEq for RemoteModel {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider && self.id == other.id && self.base_url == other.base_url
    }
}

impl Eq for RemoteModel {}

impl std::hash::Hash for RemoteModel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.provider.hash(state);
        self.id.hash(state);
        self.base_url.hash(state);
    }
}

impl std::fmt::Display for RemoteModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.provider, self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelRef {
    Local(LocalModel),
    Remote(RemoteModel),
}

impl ModelRef {
    pub fn hub(id: ModelId) -> Self {
        Self::Local(LocalModel::Hub(id))
    }

    pub fn local(uri: Uri) -> Self {
        Self::Local(LocalModel::Path(Resource::new(uri)))
    }

    pub fn remote(provider: Provider, id: ModelId) -> Self {
        Self::Remote(RemoteModel::new(provider, id))
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        if let Self::Remote(remote) = self {
            self = Self::Remote(remote.base_url(url));
        }

        self
    }

    /// The weights-bearing model, or an error naming the remote one that has none.
    pub fn local_or_err(&self) -> Result<&LocalModel> {
        match self {
            Self::Local(model) => Ok(model),
            Self::Remote(remote) => Err(Error::Load(format!("{remote} is a remote model and has no weights"))),
        }
    }
}

impl std::fmt::Display for ModelRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(model) => write!(f, "{model}"),
            Self::Remote(remote) => write!(f, "{remote}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_config_json_model_type() {
        assert_eq!("bert".parse::<Architecture>().unwrap(), Architecture::Bert);
        assert_eq!("distilbert".parse::<Architecture>().unwrap(), Architecture::DistilBert);
    }

    /// An unrecognised architecture must not be an error -- config.json may name anything.
    #[test]
    fn an_unknown_architecture_is_not_an_error() {
        assert_eq!("mamba".parse::<Architecture>().unwrap(), Architecture::Unknown);
        assert_eq!(
            serde_json::from_str::<Architecture>("\"mamba\"").unwrap(),
            Architecture::Unknown
        );
    }

    #[test]
    fn serde_round_trips() {
        assert_eq!(serde_json::to_string(&Architecture::Bert).unwrap(), "\"bert\"");
        assert_eq!(serde_json::from_str::<Architecture>("\"bart\"").unwrap(), Architecture::Bart);
    }
}
