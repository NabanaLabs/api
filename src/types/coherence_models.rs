//
// Coherence
//  

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CoherenceModels {
    CommandLight,
    CommandLightNightly,
    Command,
    CommandNightly,
}

// Constants providing additional details about each Anthropic model
impl CoherenceModels {
    const fn context_window(&self) -> usize {
        match *self {
            CoherenceModels::CommandLight | CoherenceModels::Command => 4096,
            CoherenceModels::CommandLightNightly | CoherenceModels::CommandNightly => 8192,
        }
    }

    const fn training_data(&self) -> &'static str {
        "Up to Date"
    }
}