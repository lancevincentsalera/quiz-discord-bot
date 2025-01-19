use std::sync::{Arc, Mutex};

use serenity::{all::{CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup}, model::colour};

use crate::quiz_manager::QuizManager;

pub async fn handle_results_interaction(
    ctx: &Context,
    command: CommandInteraction,
    quiz_manager_key: &Arc<Mutex<QuizManager>>,
) {
    command.defer(&ctx.http).await.expect("Failed to defer interaction");

    
    let results = {
        let mut quiz_manager = quiz_manager_key.lock().expect("Failed to lock quiz manager");
        let final_results = quiz_manager.finalize_results().expect("Failed to finalize results");
        final_results
    };
    
    command.create_followup(
        &ctx.http,
        CreateInteractionResponseFollowup::new().add_embed(
            CreateEmbed::new()
                .title("Results")
                .description(results)
                .color(colour::Colour::DARK_GREEN),
        ),
    ).await.expect("Failed to send followup");
}