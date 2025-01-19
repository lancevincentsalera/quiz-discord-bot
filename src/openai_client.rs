use rand::seq::SliceRandom;
use reqwest::Client;
use serde::Deserialize;
use serenity::json::json;
use std::env;

#[derive(Deserialize, Debug)]
pub struct OpenAIQuizResponse {
    pub id: String,
    pub choices: Vec<OpenAIQuizResponseChoices>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIQuizResponseChoices {
    pub message: OpenAIQuizResponseMessage,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIQuizResponseMessage {
    pub role: String,
    pub content: String,
}

pub async fn generate_quiz(
    difficulty: String,
) -> Result<OpenAIQuizResponse, Box<dyn std::error::Error>> {
    let api_key =
        env::var("OPENAI_API_KEY").expect("Expected an OpenAI API key in the environment");

    let topics = vec![
        "blockchain",
        "low-level programming concepts",
        "data structures and algorithms",
        "design and analysis of algorithms",
        "operating systems",
        "computer networks",
        "cryptography",
        "database systems",
        "software engineering",
        "computer architecture",
        "cardano blockchain",
        "automata theory",
        "compiler and interpreter design",
        "computer security",
        "ui/ux design",
        "web development",
        "macos/linux terminal commands",
        "Cloud computing",
        "Machine learning",
        "Artificial intelligence",
    ];

    let selected_topic = topics.choose(&mut rand::thread_rng()).unwrap();

    let prompt = format!(
        "Generate one multiple-choice question about {}.
         Difficulty: {}.
         Provide exactly 4 distinct options (a, b, c, d) and indicate and RANDOMIZE which one is correct.
         Ensure the correct answer NOT OBVIOUS.
         Choices SHOULD not be too long.
         Return it in a JSON format like this:
         {{ 
             \"question\": \"...\", 
             \"options\": {{ \"a\": \"...\", \"b\": \"...\", \"c\": \"...\", \"d\": \"...\" }}, 
             \"correct\": \"[random]\" 
         }}",
        selected_topic,
        difficulty
    );

    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await?
        .json::<OpenAIQuizResponse>()
        .await?;

    Ok(response)
}
