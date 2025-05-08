use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Sequence)]
pub enum Model {
    // Ollama models
    /// [Meta's Llama3.1](https://ollama.com/library/llama3.1:8b-instruct-q4_K_M)
    #[serde(rename = "llama3.1:8b-instruct-q4_K_M")]
    Llama3_1_8bInstructQ4Km,
    /// [Meta's LLama3.2](https://ollama.com/library/llama3.2:1b-instruct-q4_K_M)
    #[serde(rename = "llama3.2:1b-instruct-q4_K_M")]
    Llama3_2_1bInstructQ4Km,
    /// [Meta's LLama3.3](https://ollama.com/library/llama3.3:70b-instruct-q4_K_M)
    #[serde(rename = "llama3.3:70b-instruct-q4_K_M")]
    Llama3_3_70bInstructQ4Km,
    /// [Mistral's Nemo](https://ollama.com/library/mistral-nemo:12b)
    #[serde(rename = "mistral-nemo:12b")]
    MistralNemo12b,
    /// [Google's Gemma3 4b](https://ollama.com/library/gemma3:4b)
    #[serde(rename = "gemma3:4b")]
    Gemma3_4b,
    /// [Google's Gemma3 12b](https://ollama.com/library/gemma3:12b)
    #[serde(rename = "gemma3:12b")]
    Gemma3_12b,
    /// [Google's Gemma3 27b](https://ollama.com/library/gemma3:27b)
    #[serde(rename = "gemma3:27b")]
    Gemma3_27b,

    // OpenAI models
    /// [OpenAI's GPT-4o](https://platform.openai.com/docs/models#gpt-4o)
    #[serde(rename = "gpt-4o")]
    GPT4o,
    /// [OpenAI's GPT-4o mini](https://platform.openai.com/docs/models#gpt-4o-mini)
    #[serde(rename = "gpt-4o-mini")]
    GPT4oMini,

    // Gemini models
    /// [Google's Gemini 2.5 Pro experimental](https://ai.google.dev/gemini-api/docs/models#gemini-2.5-pro-preview-03-25)
    #[serde(rename = "gemini-2.5-pro-exp-03-25")]
    Gemini2_5ProExp,
    /// [Google's Gemini 2.0 Flash](https://ai.google.dev/gemini-api/docs/models#gemini-2.0-flash)
    #[serde(rename = "gemini-2.0-flash")]
    Gemini2_0Flash,

    /// OpenRouter Models
    /// [Anthropic's Claude 3.5 Sonnet](https://openrouter.ai/models?q=claude-3.5-sonnet)
    #[serde(rename = "anthropic/claude-3.5-sonnet")]
    OR3_5Sonnet,
    /// [Anthropic's Claude 3.7 Sonnet](https://openrouter.ai/models?q=claude-3.7-sonnet)
    #[serde(rename = "anthropic/claude-3-7-sonnet")]
    OR3_7Sonnet,
}

impl Model {
    /// Returns whether the model supports tool calling.
    #[deprecated]
    pub fn supports_tool_calling(&self) -> bool {
        match self {
            // OpenAI models that support tool calling
            Model::GPT4o | Model::GPT4oMini => true,
            // others do not, by default
            _ => false,
        }
    }

    /// Returns whether the model supports reasoning.
    pub fn has_reasoning(&self) -> bool {
        false
    }

    /// Returns an iterator over all models.
    #[inline(always)]
    pub fn all() -> impl Iterator<Item = Model> {
        enum_iterator::all::<Model>()
    }

    /// Returns an iterator over all models that belong to a given provider.
    #[inline(always)]
    pub fn all_with_provider(provider: &ModelProvider) -> impl Iterator<Item = Model> + '_ {
        enum_iterator::all::<Model>().filter(move |m| m.provider() == *provider)
    }

    /// Returns the provider that hosts the model.
    #[inline]
    pub fn provider(&self) -> ModelProvider {
        ModelProvider::from(self)
    }
}

impl From<Model> for String {
    fn from(model: Model) -> Self {
        model.to_string() // via Display
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // guaranteed not to fail because this is enum to string serialization
        let self_str = serde_json::to_string(&self).unwrap_or_default();

        // remove quotes from JSON
        write!(f, "{}", self_str.trim_matches('"'))
    }
}

impl TryFrom<String> for Model {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Model {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // serde requires quotes (for JSON)
        serde_json::from_str::<Self>(&format!("\"{}\"", value))
            .map_err(|e| format!("Model {} invalid: {}", value, e))
    }
}

/// A model provider is a service that hosts the chosen Model.
/// It can be derived from the model name, e.g. GPT4o is hosted by OpenAI (via API), or Phi3 is hosted by Ollama (locally).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Sequence)]
pub enum ModelProvider {
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "openrouter")]
    OpenRouter,
}

impl ModelProvider {
    /// Returns an iterator over all model providers.
    #[inline(always)]
    pub fn all() -> impl Iterator<Item = ModelProvider> {
        enum_iterator::all::<ModelProvider>()
    }

    /// Returns all models that belong to the provider.
    #[inline]
    pub fn models(&self) -> impl Iterator<Item = Model> + '_ {
        Model::all_with_provider(self)
    }
}

impl From<Model> for ModelProvider {
    fn from(value: Model) -> Self {
        Self::from(&value)
    }
}

impl From<&Model> for ModelProvider {
    fn from(model: &Model) -> Self {
        match model {
            // ollama
            Model::Gemma3_12b => ModelProvider::Ollama,
            Model::Gemma3_27b => ModelProvider::Ollama,
            Model::Gemma3_4b => ModelProvider::Ollama,
            Model::Llama3_1_8bInstructQ4Km => ModelProvider::Ollama,
            Model::Llama3_2_1bInstructQ4Km => ModelProvider::Ollama,
            Model::Llama3_3_70bInstructQ4Km => ModelProvider::Ollama,
            Model::MistralNemo12b => ModelProvider::Ollama,
            // openai
            Model::GPT4o => ModelProvider::OpenAI,
            Model::GPT4oMini => ModelProvider::OpenAI,
            // gemini
            Model::Gemini2_0Flash => ModelProvider::Gemini,
            Model::Gemini2_5ProExp => ModelProvider::Gemini,
            // openrouter
            Model::OR3_5Sonnet => ModelProvider::OpenRouter,
            Model::OR3_7Sonnet => ModelProvider::OpenRouter,
        }
    }
}

impl TryFrom<String> for ModelProvider {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        ModelProvider::try_from(value.as_str())
    }
}

impl TryFrom<&str> for ModelProvider {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // serde requires quotes (for JSON)
        serde_json::from_str::<Self>(&format!("\"{}\"", value))
            .map_err(|e| format!("Model provider {} invalid: {}", value, e))
    }
}

impl fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // guaranteed not to fail because this is enum to string serialization
        let self_str = serde_json::to_string(&self).unwrap_or_default();
        // remove quotes from JSON
        write!(f, "{}", self_str.trim_matches('"'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_string_conversion() {
        let model = Model::OR3_5Sonnet;

        // convert to string
        let model_str: String = model.clone().into();
        assert_eq!(model_str, "anthropic/claude-3.5-sonnet");

        // (try) convert from string
        let model_from = Model::try_from(model_str).expect("should convert");
        assert_eq!(model_from, model);

        // (try) convert from string
        let model = Model::try_from("this-model-does-not-will-not-exist".to_string());
        assert!(model.is_err());
    }

    #[test]
    fn test_model_string_serde() {
        let model = Model::GPT4o;

        // serialize to string via serde
        let model_str = serde_json::to_string(&model).expect("should serialize");
        assert_eq!(model_str, "\"gpt-4o\"");

        // deserialize from string via serde
        let model_from: Model = serde_json::from_str(&model_str).expect("should deserialize");
        assert_eq!(model_from, model);

        // (try) deserialize from invalid model
        let bad_model = serde_json::from_str::<Model>("\"this-model-does-not-will-not-exist\"");
        assert!(bad_model.is_err());
    }

    #[test]
    fn test_provider_string_serde() {
        let provider = ModelProvider::OpenAI;

        // serialize to string via serde
        let provider_str = serde_json::to_string(&provider).expect("should serialize");
        assert_eq!(provider_str, "\"openai\"");

        // deserialize from string via serde
        let provider_from: ModelProvider =
            serde_json::from_str(&provider_str).expect("should deserialize");
        assert_eq!(provider_from, provider);

        // (try) deserialize from invalid model
        let bad_provider =
            serde_json::from_str::<ModelProvider>("\"this-provider-does-not-will-not-exist\"");
        assert!(bad_provider.is_err());
    }

    #[test]
    fn test_model_iterator() {
        let models = Model::all().collect::<Vec<_>>();
        assert!(models.len() > 7); // arbitrary but large enough
    }

    #[test]
    fn test_model_provider_iterator() {
        let models_providers = ModelProvider::all().collect::<Vec<_>>();
        assert!(models_providers.len() > 2); // arbitrary but large enough
    }
}
