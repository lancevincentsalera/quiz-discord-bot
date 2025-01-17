
use serenity::all::ResolvedValue;
use serenity::{all::CreateCommandOption, builder::CreateCommand};
use serenity::model::application::ResolvedOption;

use crate::QuizDifficulty;

pub fn run(options: &[ResolvedOption]) -> Result<String, String> {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(difficulty), ..
    }) = options.first() 
    {
        Ok(difficulty.to_string())
    } else {
        Err("No valid difficulty provided.".to_string())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("quiz")
        .description("Generate a quiz question.")
        .add_option(
            CreateCommandOption
            ::new(serenity::all::CommandOptionType::String, "difficulty", "The difficulty of the quiz question.")
                .add_string_choice(QuizDifficulty::Easy.to_string(), QuizDifficulty::Easy.to_string())
                .add_string_choice(QuizDifficulty::Average.to_string(), QuizDifficulty::Average.to_string())
                .add_string_choice(QuizDifficulty::Hard.to_string(), QuizDifficulty::Hard.to_string())
            .required(true)
    )
}