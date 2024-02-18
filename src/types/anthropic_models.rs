//
// Anthropic
//

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnthropicModels {
    ClaudeInstant,
    Claude2,
    Claude2_1,
}

// Constants providing additional details about each Anthropic model
impl AnthropicModels {
    pub const fn context_window(&self) -> usize {
        match *self {
            AnthropicModels::ClaudeInstant => 100000,
            AnthropicModels::Claude2 => 100000,
            AnthropicModels::Claude2_1 => 200000,
        }
    }

    pub const fn training_data(&self) -> &'static str {
        "Up to Dec 2022"
    }
}