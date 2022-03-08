use std::{collections::HashMap, env, sync::Mutex, time::Duration};

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::TypeMapKey;
use sqlx::PgPool;
use tracing::{error, info};

use crate::db::get_db_pool;

mod db;
mod structures;
mod subcommands;

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}

pub struct Data {
    pool: PgPool,
}

//type CommandResult = Result<(), Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command, slash_command, track_edits)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or_else(|| ctx.author());
    ctx.say(format!(
        "{}'s account was created at {}",
        user.name,
        user.created_at()
    ))
    .await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is the poise error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[poise::command(prefix_command, track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "\
Built by Zalgo#1649 using Rust with the Serenity and Poise libraries.",
            show_context_menu_commands: true,
            ephemeral: true,
        },
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load env file");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db_creds = env::var("DATABASE_URL").expect("Expected database URL in environment");
    let pool = get_db_pool(db_creds).await.expect("bad DB URL");
    poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data { pool }) }))
        .options(poise::FrameworkOptions {
            // configure framework here
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),

            commands: vec![
                age(),
                help(),
                register(),
                poise::Command {
                    subcommands: vec![
                        subcommands::add(),
                        subcommands::remove(),
                        //parent?
                    ],
                    ..subcommands::greeting()
                },
            ],
            ..Default::default()
        })
        .run()
        .await
        .unwrap();
}
