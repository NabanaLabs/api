//
// OpenAI
//

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OpenAIModels {
    // GPT-4 models
    GPT4,
    GPT4Turbo,
    GPT4_1106,
    GPT4Vision,

    // GPT-3.5 Turbo models
    GPT3_5Turbo0125,
    GPT3_5Turbo,
    GPT3_5Turbo1106,
    GPT3_5TurboInstruct,
    GPT3_5Turbo16k,
    GPT3_5Turbo0613,
    GPT3_5Turbo16k0613,

    // GPT base models
    Babbage002,
    Davinci002,
}

impl OpenAIModels {
    pub const fn context_window(&self) -> usize {
        match *self {
            OpenAIModels::GPT4 | OpenAIModels::GPT4Turbo | OpenAIModels::GPT4_1106 | OpenAIModels::GPT4Vision => 128000,
            OpenAIModels::GPT3_5Turbo0125 | OpenAIModels::GPT3_5Turbo | OpenAIModels::GPT3_5Turbo1106 | OpenAIModels::GPT3_5TurboInstruct | OpenAIModels::GPT3_5Turbo16k | OpenAIModels::GPT3_5Turbo0613 | OpenAIModels::GPT3_5Turbo16k0613 => 16385,
            OpenAIModels::Babbage002 | OpenAIModels::Davinci002 => 16384,
        }
    }

    pub const fn training_data(&self) -> &'static str {
        match *self {
            OpenAIModels::GPT4 | OpenAIModels::GPT4Turbo | OpenAIModels::GPT4_1106 | OpenAIModels::GPT4Vision => "Up to Apr 2023",
            OpenAIModels::GPT3_5Turbo0125 | OpenAIModels::GPT3_5Turbo | OpenAIModels::GPT3_5Turbo1106 | OpenAIModels::GPT3_5TurboInstruct | OpenAIModels::GPT3_5Turbo16k | OpenAIModels::GPT3_5Turbo0613 | OpenAIModels::GPT3_5Turbo16k0613 => "Up to Sep 2021",
            OpenAIModels::Babbage002 | OpenAIModels::Davinci002 => "Up to Sep 2021",
        }
    }
}