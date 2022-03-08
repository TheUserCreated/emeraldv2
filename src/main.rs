use std::{collections::HashMap, env, sync::Mutex, time::Duration};

use poise::serenity_prelude as serenity;
use tracing::{error, info};

mod db;
mod structures;
mod subcommands;

pub struct Data {}

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

    poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .options(poise::FrameworkOptions {
            // configure framework here
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                ..Default::default()
            },
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
