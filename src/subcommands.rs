use poise::serenity_prelude::model::guild::Member;
use poise::serenity_prelude::Context as serenityContext;
use poise::serenity_prelude::{ChannelId, Mentionable, Role};
use poise::Modal;
use tracing::log::error;

use crate::db::{
    get_all_greeting, remove_greeting_by_channel, remove_greeting_by_role,
    remove_greeting_internal, set_greeting_internal,
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

#[poise::command(slash_command)]
pub async fn add(
    ctx: ApplicationContext<'_>,
    #[description = "Role to greet on"] role: poise::serenity_prelude::Role,
    #[description = "Channel to send the greeting in"] channel: poise::serenity_prelude::Channel,
) -> Result<(), Error> {
    let greetdata: GreetingModal = GreetingModal::execute(ctx).await.unwrap();

    let greeting = greetdata.greeting_input;

    let pool = {
        let data = ctx.data;
        &data.pool
    };
    set_greeting_internal(pool, &role.guild_id, channel.id(), role.id, greeting).await?;

    let response = format!(
        "Greeting for role {} added in channel {} !",
        role.name,
        channel.mention()
    );
    poise::say_reply(ctx.into(), response).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn list(
    ctx: ApplicationContext<'_>,
    #[description = "Role whose greetings you'd like to list."] role: Option<
        poise::serenity_prelude::Role,
    >,
    #[description = "List all greetings for a given channel."] channel: Option<
        poise::serenity_prelude::Channel,
    >,
) -> Result<(), Error> {
    let pool = {
        let data = ctx.data;
        &data.pool
    };
    let guild_id = ctx.interaction.guild_id().unwrap();
    let greetings_list = get_all_greeting(pool, &guild_id).await?;
    let mut response: String = "".to_string();
    let mut index = 0;
    let max_index = greetings_list.len() as i32;
    for item in greetings_list {
        index += 1;
        if index == max_index {
            let individual_response = format!(
                "Greeting for role <@&{}> in channel <#{}> with text `{}`",
                item.role_id_internal, item.channel_id_internal, item.greeting_text
            );
            response += individual_response.as_str();
        } else {
            let individual_response = format!(
                "Greeting for role <@&{}> in channel <#{}> with text `{}`\n",
                item.role_id_internal, item.channel_id_internal, item.greeting_text
            );
            response += individual_response.as_str();
        }
    }

    poise::say_reply(ctx.into(), response).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(
    ctx: ApplicationContext<'_>,
    #[description = "Role whose greeting you'd like to remove"] role: poise::serenity_prelude::Role,
    #[description = "Channel the greeting is in"] channel: poise::serenity_prelude::Channel,
) -> Result<(), Error> {
    let pool = {
        let data = ctx.data;
        &data.pool
    };

    let remove_result =
        remove_greeting_internal(&(pool), &role.guild_id, &channel.id(), &role.id).await;
    if remove_result.is_ok() {
        let response = format!(
            "Greeting for role {} removed from channel {} !",
            role.name,
            channel.mention()
        );
        poise::say_reply(ctx.into(), response).await?;
    } else {
        error!(
            "Greeting removal erred with error: {:?}",
            remove_result.err().unwrap()
        );
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn replacement(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let response = "Certain text in greetings is replaced with specific information. The list is as follows: \n\
    `%servername%` is replaced with the name of the server.\n\
    `%channelmention%` is replaced with the channel the greeting is sent in.\n\
    `%rolename%` mentions the role name (*without* pinging it). \n\
    `%usermention%` mentions the user being greeted.".to_string();
    poise::say_reply(ctx.into(), response)
        .await
        .expect("couldn't reply to a command!");
    Ok(())
}

// This entire function is ugly. Too bad! (if you know a better way to do any of this, please submit a PR or create an issue.)
pub async fn greeting_replacements(
    ctx: &serenityContext,
    greeting: String,
    role: &Role,
    member: &Member,
    channel_id: &ChannelId,
) -> Result<String, Error> {
    let guild = member.guild_id;
    let guild = ctx
        .http
        .get_guild(guild.0)
        .await
        .expect("got a greeting for a guild i can't fetch");
    let finished_text = greeting.replace("%servername%", guild.name.as_str());
    let finished_text =
        finished_text.replace("%usermention%", member.mention().to_string().as_str());
    let finished_text = finished_text.replace(
        "%channelmention%",
        ctx.http
            .get_channel(channel_id.0)
            .await
            .expect("can't send a greeting to a channel i cant speak in or see!")
            .mention()
            .to_string()
            .as_str(),
    );

    let finished_text = finished_text.replace("%rolename%", role.name.as_str());

    Ok(finished_text)
}
