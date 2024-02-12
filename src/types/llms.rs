use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{anthropic_models::AnthropicModels, coherence_models::CoherenceModels, openai_models::OpenAIModels};

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