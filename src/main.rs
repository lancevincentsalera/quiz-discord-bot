use std::sync::Arc;
use std::{env, vec};

use openai_client::generate_quiz;
use quiz_manager::{QuizManager, QuizQuestion};
use serenity::all::{
    CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, GuildId,
};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands;
mod openai_client;
mod quiz_manager;

#[derive(Debug)]
pub enum QuizDifficulty {
    Easy,
    Average,
    Hard,
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

        println!("Commands: {:?}", commands);
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        println!("Received an interaction: {:?}", interaction.kind());

        if let Interaction::Command(command) = interaction {
            let command_name = &command.data.name;
            println!("Received command: {:?}", command_name);

            let data = _ctx.data.read().await;
            let quiz_manager_lock = data
                .get::<QuizManagerKey>()
                .expect("Expected QuizManager in TypeMap")
                .clone();

            let content = match command_name.as_str() {
                "quiz" => {
                    command
                        .defer(&_ctx.http)
                        .await
                        .expect("Error deferring command");

                    let result_embed = {
                        let mut quiz_manager = quiz_manager_lock.lock().await;

                        let embed = if quiz_manager.current_quiz.is_some() {
                            CreateEmbed::new()
                                .title("Error")
                                .description("A quiz is already active.")
                                .color(0xff0000) // Red color
                        } else {
                            let difficulty = commands::quiz::run(&command.data.options())
                                .expect("Error running quiz command");

                            let quiz_generated = generate_quiz(difficulty.clone()).await.unwrap();
                            let quiz_question = quiz_generated.choices[0].message.content.clone();

                            let cleaned_json = quiz_question
                                .trim_start_matches("```json")
                                .trim_end_matches("```")
                                .trim();
                            println!("Raw quiz_question: {}", cleaned_json);

                            // Safer deserialization with error handling
                            let quiz_json =
                                match serde_json::from_str::<QuizQuestion>(&cleaned_json) {
                                    Ok(parsed) => {
                                        quiz_manager.set_quiz(parsed.clone());
                                        parsed
                                    }
                                    Err(e) => {
                                        println!("JSON Parse Error: {}", e);
                                        println!("Received content: {}", cleaned_json);
                                        return command
                                            .create_response(&_ctx.http, {
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content("Error parsing quiz question"),
                                                )
                                            })
                                            .await
                                            .expect("Error sending response");
                                    }
                                };

                            let embed = CreateEmbed::default()
                                .title("Quiz Time!")
                                .description(format!("**Difficulty:** *{}*", difficulty))
                                .color(0x00ff00) // Green color
                                .field(
                                    "Question",
                                    format!(
                                        "**{}**\n\n**Options**:\nA. {}\nB. {}\nC. {}\nD. {}",
                                        quiz_json.question,
                                        quiz_json.options.a,
                                        quiz_json.options.b,
                                        quiz_json.options.c,
                                        quiz_json.options.d
                                    ),
                                    false,
                                )
                                .footer(CreateEmbedFooter::new("Type `/answer` to respond."));
                            embed
                        };
                        embed
                    };

                    command.create_followup(
                        &_ctx,
                        CreateInteractionResponseFollowup::new().add_embed(result_embed),
                    )
                }
                "answer" => {
                    command
                        .defer_ephemeral(&_ctx.http)
                        .await
                        .expect("Error deferring command");
                    let answer = commands::answer::run(&command.data.options()).unwrap();
                    let user_id = u64::from(command.user.id);

                    let answer_submission_response = {
                        let mut quiz_manager = quiz_manager_lock.lock().await;
                        let resp = if quiz_manager.current_quiz.is_some() {
                            if quiz_manager.has_user_answered(user_id) {
                                CreateEmbed::new()
                                    .title("Error")
                                    .description(
                                        "You have already submitted an answer for this quiz.",
                                    )
                                    .color(0xff0000) // Red color
                            } else {
                                quiz_manager.set_answer(user_id, answer);
                                CreateEmbed::new()
                                    .title("Answer Submitted")
                                    .description(format!("Your answer: **{}**", answer))
                                    .color(0x00ff00) // Green color
                            }
                        } else {
                            CreateEmbed::new()
                                .title("Error")
                                .description("No active quiz found.")
                                .color(0xff0000) // Red color
                        };

                        resp
                    };

                    command.create_followup(
                        &_ctx,
                        CreateInteractionResponseFollowup::new()
                            .add_embed(answer_submission_response),
                    )
                }
                "results" => {
                    command
                        .defer(&_ctx.http)
                        .await
                        .expect("Error deferring command");
                    let results = {
                        let mut quiz_manager = quiz_manager_lock.lock().await;
                        quiz_manager.finalize_results().unwrap()
                    };

                    command.create_followup(
                        &_ctx,
                        CreateInteractionResponseFollowup::new()
                            .add_embed(CreateEmbed::new().description(results)),
                    )
                }
                _ => {
                    command
                        .defer(&_ctx.http)
                        .await
                        .expect("Error deferring command");
                    command.create_followup(
                        &_ctx,
                        CreateInteractionResponseFollowup::new().add_embed(
                            CreateEmbed::new()
                                .title("Error")
                                .description("Invalid command")
                                .color(0xff0000),
                        ),
                    ) // Red color
                }
            };

            if let Err(why) = content.await {
                println!("Error sending response: {:?}", why);
            }
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
