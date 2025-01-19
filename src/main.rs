use std::sync::{Arc, Mutex};
use std::{env, vec};

use quiz_manager::QuizManager;
use serenity::all::{
    CreateCommand, GuildId,
};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands;
mod openai_client;
mod quiz_manager;
mod interactions;



pub struct QuizManagerKey;

impl TypeMapKey for QuizManagerKey {
    type Value = Arc<Mutex<QuizManager>>;
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected a guild id in the environment")
                .parse()
                .expect("The guild id is not a valid integer"),
        );

        let commands = guild_id
            .set_commands(
                &_ctx.http,
                vec![
                    commands::quiz::register(),
                    commands::answer::register(),
                    CreateCommand::new("results").description("Get the results of the quiz."),
                ],
            )
            .await;
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        println!("Received an interaction: {:?}", interaction.kind());

        if let Interaction::Command(command) = interaction {
            interactions::interaction_handlers::handle_all_interactions(_ctx, command).await;
        }
    }
}

// this is the entry point for the discord bot program
// that will do ai generation quizzes about
// blockchain and low-level general programming concepts
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::empty();

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<QuizManagerKey>(Arc::new(Mutex::new(QuizManager::new())));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
