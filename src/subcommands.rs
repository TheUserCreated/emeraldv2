use std::borrow::Borrow;

use poise::serenity_prelude::{Mentionable, Role};
use poise::Modal;
use sqlx::PgPool;
use tracing::info;

use crate::db::{
    remove_greeting_by_channel, remove_greeting_by_role, remove_greeting_internal,
    set_greeting_internal,
};
use crate::{ConnectionPool, Context, Data, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn greeting(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}

type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;

#[derive(Debug, Modal)]
#[name = "Greeting text"]
struct GreetingModal {
    #[name = "The text for your greeting"] // Field name
    #[paragraph] // Multi-line input
    greeting_input: String,
}

#[poise::command(slash_command, track_edits)]
pub async fn add(
    ctx: ApplicationContext<'_>,
    #[description = "Role to greet on"] role: poise::serenity_prelude::Role,
    #[description = "Channel to send the greeting in"] channel: poise::serenity_prelude::Channel,
) -> Result<(), Error> {
    let greetdata: GreetingModal = GreetingModal::execute(ctx).await?;
    let greeting = greetdata.greeting_input;

    let greeting = greeting_replacements(ctx, greeting, role.borrow())
        .await
        .expect("greeting replacement failed!");
    let data = ctx.data;
    let pool = &data.pool;
    set_greeting_internal(
        pool,
        &ctx.interaction.guild_id().unwrap(),
        channel.id(),
        role.id,
        greeting,
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked `remove`!").await?;
    Ok(())
}

// This entire function is ugly. Too bad! (if you know a better way to do any of this, please submit a PR or create an issue.)
pub async fn greeting_replacements(
    ctx: ApplicationContext<'_>,
    greeting: String,
    role: &Role,
) -> Result<String, Error> {
    let interaction_data = ctx.interaction;
    let member = interaction_data.member().unwrap();
    let guild = interaction_data.member().unwrap().guild_id;
    let guild = ctx.discord.http.get_guild(guild.0).await.unwrap().name;
    let finished_text = greeting.replace("%servername%", guild.as_str());
    let finished_text =
        finished_text.replace("%usermention%", member.mention().to_string().as_str());
    let finished_text = finished_text.replace(
        "%channelmention%",
        ctx.discord
            .http
            .get_channel(interaction_data.channel_id().0)
            .await
            .unwrap()
            .mention()
            .to_string()
            .as_str(),
    );
    let finished_text = finished_text.replace("%rolename%", role.name.as_str());

    Ok(finished_text)
}
