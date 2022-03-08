use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn greeting(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Role to greet on"] role: poise::serenity_prelude::Role,
) -> Result<(), Error> {
    let response = format!("You invoked `add` with role {}", role.name.as_str());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn remove(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked `remove`!").await?;
    Ok(())
}
