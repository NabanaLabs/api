use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{anthropic_models::AnthropicModels, coherence_models::CoherenceModels, openai_models::OpenAIModels};

#[derive(Default, Debug, Serialize)]
pub struct ModelInfo {
    pub company: Option<String>,
    pub model: String,
    pub context_window: usize,
    pub training_data: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LLMs {
    None,
    OpenAIModels(OpenAIModels),
    AnthropicModels(AnthropicModels),
    CoherenceModels(CoherenceModels),
}

impl LLMs {
    pub fn to_string(&self) -> String {
        match self {
            LLMs::None => String::from("none"),
            LLMs::OpenAIModels(OpenAIModels::GPT4) => String::from("gpt-4"),
            LLMs::OpenAIModels(OpenAIModels::GPT4Turbo) => String::from("gpt-4-turbo"),
            LLMs::OpenAIModels(OpenAIModels::GPT4_1106) => String::from("gpt-4-1106"),
            LLMs::OpenAIModels(OpenAIModels::GPT4Vision) => String::from("gpt-4-vision"),
            LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo0125) => String::from("gpt-3.5-turbo-0125"),
            LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo) => String::from("gpt-3.5-turbo"),
            LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo1106) => String::from("gpt-3.5-turbo-1106"),
            LLMs::OpenAIModels(OpenAIModels::GPT3_5TurboInstruct) => String::from("gpt-3.5-turbo-instruct"),
            LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo16k) => String::from("gpt-3.5-turbo-16k"),
            LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo0613) => String::from("gpt-3.5-turbo-0613"),
            LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo16k0613) => String::from("gpt-3.5-turbo-16k-0613"),
            LLMs::OpenAIModels(OpenAIModels::Babbage002) => String::from("babbage-002"),
            LLMs::OpenAIModels(OpenAIModels::Davinci002) => String::from("davinci-002"),

            LLMs::AnthropicModels(AnthropicModels::ClaudeInstant) => String::from("claude-instant-1.2"),
            LLMs::AnthropicModels(AnthropicModels::Claude2) => String::from("claude-2"),
            LLMs::AnthropicModels(AnthropicModels::Claude2_1) => String::from("claude-2.1"),

            LLMs::CoherenceModels(CoherenceModels::CommandLight) => String::from("command-light"),
            LLMs::CoherenceModels(CoherenceModels::CommandLightNightly) => String::from("command-light-nightly"),
            LLMs::CoherenceModels(CoherenceModels::Command) => String::from("command"),
            LLMs::CoherenceModels(CoherenceModels::CommandNightly) => String::from("command-nightly"),
        }
    }

    pub fn coherence_models_info() -> Vec<ModelInfo> {
        return vec![
            ModelInfo {
                company: Some(String::from("Coherence")),
                model: LLMs::CoherenceModels(CoherenceModels::CommandLight).to_string(),
                context_window: CoherenceModels::CommandLight.context_window(),
                training_data: CoherenceModels::CommandLight.training_data(),
            },
            ModelInfo {
                company: Some(String::from("Coherence")),
                model: LLMs::CoherenceModels(CoherenceModels::CommandLightNightly).to_string(),
                context_window: CoherenceModels::CommandLightNightly.context_window(),
                training_data: CoherenceModels::CommandLightNightly.training_data(),
            },
            ModelInfo {
                company: Some(String::from("Coherence")),
                model: LLMs::CoherenceModels(CoherenceModels::Command).to_string(),
                context_window: CoherenceModels::Command.context_window(),
                training_data: CoherenceModels::Command.training_data(),
            },
            ModelInfo {
                company: Some(String::from("Coherence")),
                model: LLMs::CoherenceModels(CoherenceModels::CommandNightly).to_string(),
                context_window: CoherenceModels::CommandNightly.context_window(),
                training_data: CoherenceModels::CommandNightly.training_data(),
            },
        ];
    }

    pub fn anthropic_models_info() -> Vec<ModelInfo> {
        return vec![
            ModelInfo {
                company: Some(String::from("Anthropic")),
                model: LLMs::AnthropicModels(AnthropicModels::ClaudeInstant).to_string(),
                context_window: AnthropicModels::ClaudeInstant.context_window(),
                training_data: AnthropicModels::ClaudeInstant.training_data(),
            },
            ModelInfo {
                company: Some(String::from("Anthropic")),
                model: LLMs::AnthropicModels(AnthropicModels::Claude2).to_string(),
                context_window: AnthropicModels::Claude2.context_window(),
                training_data: AnthropicModels::Claude2.training_data(),
            },
            ModelInfo {
                company: Some(String::from("Anthropic")),
                model: LLMs::AnthropicModels(AnthropicModels::Claude2_1).to_string(),
                context_window: AnthropicModels::Claude2_1.context_window(),
                training_data: AnthropicModels::Claude2_1.training_data(),
            },
        ];
    }

    pub fn openai_models_info() -> Vec<ModelInfo> {
        return vec![
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT4).to_string(),
                context_window: OpenAIModels::GPT4.context_window(),
                training_data: OpenAIModels::GPT4.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT4Turbo).to_string(),
                context_window: OpenAIModels::GPT4Turbo.context_window(),
                training_data: OpenAIModels::GPT4Turbo.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT4_1106).to_string(),
                context_window: OpenAIModels::GPT4_1106.context_window(),
                training_data: OpenAIModels::GPT4_1106.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT4Vision).to_string(),
                context_window: OpenAIModels::GPT4Vision.context_window(),
                training_data: OpenAIModels::GPT4Vision.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo0125).to_string(),
                context_window: OpenAIModels::GPT3_5Turbo0125.context_window(),
                training_data: OpenAIModels::GPT3_5Turbo0125.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo).to_string(),
                context_window: OpenAIModels::GPT3_5Turbo.context_window(),
                training_data: OpenAIModels::GPT3_5Turbo.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo1106).to_string(),
                context_window: OpenAIModels::GPT3_5Turbo1106.context_window(),
                training_data: OpenAIModels::GPT3_5Turbo1106.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT3_5TurboInstruct).to_string(),
                context_window: OpenAIModels::GPT3_5TurboInstruct.context_window(),
                training_data: OpenAIModels::GPT3_5TurboInstruct.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo16k).to_string(),
                context_window: OpenAIModels::GPT3_5Turbo16k.context_window(),
                training_data: OpenAIModels::GPT3_5Turbo16k.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo0613).to_string(),
                context_window: OpenAIModels::GPT3_5Turbo0613.context_window(),
                training_data: OpenAIModels::GPT3_5Turbo0613.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo16k0613).to_string(),
                context_window: OpenAIModels::GPT3_5Turbo16k0613.context_window(),
                training_data: OpenAIModels::GPT3_5Turbo16k0613.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::Babbage002).to_string(),
                context_window: OpenAIModels::Babbage002.context_window(),
                training_data: OpenAIModels::Babbage002.training_data(),
            },
            ModelInfo {
                company: Some(String::from("OpenAI")),
                model: LLMs::OpenAIModels(OpenAIModels::Davinci002).to_string(),
                context_window: OpenAIModels::Davinci002.context_window(),
                training_data: OpenAIModels::Davinci002.training_data(),
            },
        ];
    }

    pub fn all_models_info() -> Vec<ModelInfo> {
        let mut all_models_info = LLMs::openai_models_info();
        all_models_info.extend(LLMs::anthropic_models_info());
        all_models_info.extend(LLMs::coherence_models_info());
        return all_models_info;
    }
}

impl FromStr for LLMs {
    type Err = ();

    fn from_str(s: &str) -> Result<LLMs, Self::Err> {
        match s {
            "none" => Ok(LLMs::None),
            "gpt-4" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT4)),
            "gpt-4-turbo" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT4Turbo)),
            "gpt-4-1106" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT4_1106)),
            "gpt-4-vision" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT4Vision)),
            "gpt-3.5-turbo-0125" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo0125)),
            "gpt-3.5-turbo" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo)),
            "gpt-3.5-turbo-1106" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo1106)),
            "gpt-3.5-turbo-instruct" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT3_5TurboInstruct)),
            "gpt-3.5-turbo-16k" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo16k)),
            "gpt-3.5-turbo-0613" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo0613)),
            "gpt-3.5-turbo-16k-0613" => Ok(LLMs::OpenAIModels(OpenAIModels::GPT3_5Turbo16k0613)),
            "babbage-002" => Ok(LLMs::OpenAIModels(OpenAIModels::Babbage002)),
            "davinci-002" => Ok(LLMs::OpenAIModels(OpenAIModels::Davinci002)),

            "claude-instant-1.2" => Ok(LLMs::AnthropicModels(AnthropicModels::ClaudeInstant)),
            "claude-2" => Ok(LLMs::AnthropicModels(AnthropicModels::Claude2)),
            "claude-2.1" => Ok(LLMs::AnthropicModels(AnthropicModels::Claude2_1)),

            "command-light" => Ok(LLMs::CoherenceModels(CoherenceModels::CommandLight)),
            "command-light-nightly" => Ok(LLMs::CoherenceModels(CoherenceModels::CommandLightNightly)),
            "command" => Ok(LLMs::CoherenceModels(CoherenceModels::Command)),
            "command-nightly" => Ok(LLMs::CoherenceModels(CoherenceModels::CommandNightly)),

            _ => Ok(LLMs::None),
        }
    }
}