use serde::{Deserialize, Serialize};

use super::organization::ModelObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub label: String,
    pub description: String,
    pub model: ModelObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    pub text: String,
    pub exact: bool,
    pub use_cosine_similarity: bool,
    pub cosine_similarity_temperature: f32,
    pub model: ModelObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Router {
    // info
    pub id: String,
    pub name: String,
    pub description: String,

    // state
    pub active: bool,
    pub deleted: bool,

    // settings

    pub max_prompt_length: i32,

    // do nothing haha
    pub use_single_model: bool,
    pub model: ModelObject,

    // https://github.com/NabanaLabs/albert-prompt-classification
    pub use_prompt_calification_model: bool,
    pub prompt_calification_model_version: String,
    pub prompt_calification_model_categories: Vec<Category>,

    // Example 
    // [Sentence {
    //    text: "code a calculator in python",
    //    exact: false,
    //    use_cosine_similarity: true,
    //    cosine_similarity_temperature: 0.5,
    //    model: ModelObject {
    //        id: "random",
    //        model_name: "gpt-3",
    //        display_name: "GPT-3",
    //        description: "The third version of the Generative Pre-trained Transformer language model developed by OpenAI.",
    //        owner: OpenAI,
    //    },
    // }]
    pub use_sentence_matching: bool,
    pub sentences: Vec<Sentence>,
}
