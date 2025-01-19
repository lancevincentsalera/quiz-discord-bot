use std::sync::{Arc, Mutex};

use crate::{commands::answer, quiz_manager::QuizManager};
use serenity::{
    all::{CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup},
    model::colour,
};

pub async fn handle_answer_interaction(
    ctx: &Context,
    command: CommandInteraction,
    quiz_manager_key: &Arc<Mutex<QuizManager>>,
) {
    command
        .defer_ephemeral(&ctx.http)
        .await
        .expect("Failed to defer interaction");
    let embed = {
        let mut quiz_manager = quiz_manager_key
            .lock().expect("Failed to lock quiz manager");

        let answer = answer::run(&command.data.options()).expect("Failed to get answer");
        let user_id = u64::from(command.user.id);

        let answer_embed = if quiz_manager.current_quiz.is_some() {
            if quiz_manager.has_user_answered(user_id) {
                CreateEmbed::new()
                    .title("Error")
                    .description("You have already answered this question.")
                    .color(colour::Colour::RED)
            } else {
                quiz_manager.set_answer(user_id, answer);

                CreateEmbed::new()
                    .title("Answer Received")
                    .description(format!("Your answer: {}", answer))
                    .color(colour::Colour::DARK_GREEN)
            }
        } else {
            CreateEmbed::new()
                .title("Error")
                .description("No active quiz found.")
                .color(colour::Colour::RED)
        };
        
        answer_embed
    };
    command
        .create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new().add_embed(embed),
        )
        .await
        .expect("Failed to send followup");
}
