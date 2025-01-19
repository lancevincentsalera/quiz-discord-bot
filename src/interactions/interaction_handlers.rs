use serenity::all::{CommandInteraction, Context};
use std::str;

use crate::QuizManagerKey;

use super::{
    answer_handler::handle_answer_interaction, quiz_handler::handle_quiz_interaction,
    results_handler::handle_results_interaction,
};

#[derive(Debug)]
pub enum QuizDifficulty {
    Easy,
    Average,
    Hard,
}

pub enum QuizCommands {
    Quiz,
    Answer,
    Results,
    Leaderboard,
}

impl ToString for QuizDifficulty {
    fn to_string(&self) -> String {
        match self {
            QuizDifficulty::Easy => String::from("easy"),
            QuizDifficulty::Average => String::from("average"),
            QuizDifficulty::Hard => String::from("hard"),
        }
    }
}

impl ToString for QuizCommands {
    fn to_string(&self) -> String {
        match self {
            QuizCommands::Quiz => String::from("quiz"),
            QuizCommands::Answer => String::from("answer"),
            QuizCommands::Results => String::from("results"),
            QuizCommands::Leaderboard => String::from("leaderboard"),
        }
    }
}

impl QuizCommands {
    fn from_str(command: &str) -> Option<Self> {
        match command {
            "quiz" => Some(Self::Quiz),
            "answer" => Some(Self::Answer),
            "results" => Some(Self::Results),
            "leaderboard" => Some(Self::Leaderboard),
            _ => None,
        }
    }
}

pub async fn handle_all_interactions(ctx: Context, command: CommandInteraction) {
    let command_name = &command.data.name;

    let quiz_manager_key = {
        let ctx_data_read = &ctx.data.read().await;
        ctx_data_read
            .get::<QuizManagerKey>()
            .expect("Failed to get quiz manager key")
            .clone()
    };

    let interaction_result = match QuizCommands::from_str(command_name.as_str()) {
        Some(QuizCommands::Quiz) => {
            handle_quiz_interaction(&ctx, command, &quiz_manager_key).await;
        }
        Some(QuizCommands::Answer) => {
            handle_answer_interaction(&ctx, command, &quiz_manager_key).await;
        }
        Some(QuizCommands::Results) => {
            handle_results_interaction(&ctx, command, &quiz_manager_key).await;
        }
        Some(QuizCommands::Leaderboard) => {
            // handle_leaderboard_interaction(ctx, command, quiz_manager_key).await;
        }
        None => {}
    };

    interaction_result
}
