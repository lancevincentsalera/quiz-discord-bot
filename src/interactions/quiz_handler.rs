use std::sync::{Arc, Mutex};

use serenity::{
    all::{
        CommandInteraction, Context, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    model::colour,
};

use crate::{
    commands::quiz,
    openai_quiz_client::generate_quiz,
    quiz_manager::{QuizManager, QuizQuestion},
};

pub async fn handle_quiz_interaction(
    ctx: &Context,
    command: CommandInteraction,
    quiz_manager_key: &Arc<Mutex<QuizManager>>,
) {
    command
        .defer(&ctx.http)
        .await
        .expect("Failed to defer interaction");
    let current_quiz = {
        let quiz_manager = quiz_manager_key
            .lock()
            .expect("Failed to lock quiz manager");
        quiz_manager.current_quiz.as_ref().cloned()
    };

    if current_quiz.is_some() {
        return command
            .create_response(&ctx.http, {
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("There is already an active quiz."),
                )
            })
            .await
            .expect("Failed to send response");
    }

    let difficulty = quiz::run(&command.data.options()).expect("Failed to get difficulty");

    let generated_quiz = generate_quiz(difficulty.clone())
        .await
        .expect("Failed to generate quiz");
    let untrimmed_quiz = generated_quiz.choices[0].message.content.clone();
    let trimmed_quiz = untrimmed_quiz
        .trim_start_matches("```json")
        .trim_end_matches("```")
        .trim();

    let deserialized_quiz = match serde_json::from_str::<QuizQuestion>(&trimmed_quiz) {
        Ok(quiz) => quiz,
        Err(e) => {
            println!("JSON Parse Error: {}", e);
            println!("Received content: {}", trimmed_quiz);
            return command
                .create_response(&ctx.http, {
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Error parsing quiz question"),
                    )
                })
                .await
                .expect("Error sending response");
        }
    };

    {
        let mut quiz_manager = quiz_manager_key
            .lock()
            .expect("Failed to lock quiz manager");
        quiz_manager.set_quiz(deserialized_quiz.clone());
    }

    let final_embed = CreateEmbed::default()
        .title("Quiz Time!")
        .description(format!("**Difficulty** : {}", difficulty))
        .color(colour::Colour::DARK_GREEN)
        .field(
            "Question",
            format!(
                "**{}**\n\n**Options**:\nA. {}\nB. {}\nC. {}\nD. {}",
                deserialized_quiz.question,
                deserialized_quiz.options.a,
                deserialized_quiz.options.b,
                deserialized_quiz.options.c,
                deserialized_quiz.options.d
            ),
            false,
        )
        .footer(CreateEmbedFooter::new("Type `/answer` to respond."));

    command
        .create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new().add_embed(final_embed),
        )
        .await
        .expect("Failed to send followup");
}
