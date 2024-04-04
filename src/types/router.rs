use mongodb::bson::{doc, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub label: String,
    pub description: String,
    pub model_id: String,
}

impl Into<Bson> for Category {
    fn into(self) -> Bson {
        doc! {
            "label": self.label,
            "description": self.description,
            "model_id": self.model_id,
        }
        .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    pub text: String,
    pub exact: bool,
    pub use_cosine_similarity: bool,
    pub cosine_similarity_temperature: f32,
    pub model_id: String,
}

impl Into<Bson> for Sentence {
    fn into(self) -> Bson {
        doc! {
            "text": self.text,
            "exact": self.exact,
            "use_cosine_similarity": self.use_cosine_similarity,
            "cosine_similarity_temperature": self.cosine_similarity_temperature,
            "model_id": self.model_id,
        }
        .into() // Convert the document into a Bson value
    }
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
    pub model_id: String,

    // https://github.com/NabanaLabs/albert-prompt-classification
    pub use_prompt_calification_model: bool,
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

impl Into<Bson> for Router {
    fn into(self) -> Bson {
        // Convert your Router struct into a BSON document
        doc! {
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "active": self.active,
            "deleted": self.deleted,
            "max_prompt_length": self.max_prompt_length,
            "use_single_model": self.use_single_model,
            "model_id": self.model_id,
            "use_prompt_calification_model": self.use_prompt_calification_model,
            "prompt_calification_model_categories": self.prompt_calification_model_categories,
            "use_sentence_matching": self.use_sentence_matching,
            "sentences": self.sentences,
        }
        .into() // Convert the document into a Bson value
    }
}