use serenity::all::ResolvedValue;
use serenity::model::application::ResolvedOption;
use serenity::{all::CreateCommandOption, builder::CreateCommand};

pub fn run(options: &[ResolvedOption]) -> Result<char, String> {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(raw_choice),
        ..
    }) = options.first()
    {
        Ok(raw_choice.chars().next().unwrap())
    } else {
        Err("No valid choice provided.".to_string())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("answer")
        .description("Answer the current quiz question.")
        .add_option(
            CreateCommandOption::new(
                serenity::all::CommandOptionType::String,
                "choice",
                "The choice you want to make.",
            )
            .add_string_choice("A", "A")
            .add_string_choice("B", "B")
            .add_string_choice("C", "C")
            .add_string_choice("D", "D")
            .required(true),
        )
}
