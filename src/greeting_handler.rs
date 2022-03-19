use std::borrow::Borrow;
use std::collections::HashSet;

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Context, Error, Member};
use tracing::log::debug;

use crate::db::get_greeting;
use crate::subcommands::greeting_replacements;
use crate::{ConnectionPool, Data};

pub async fn greeting_handler(
    ctx: &serenity::Context,
    old_if_available: &Option<Member>,
    new: &Member,
    user_data: &Data,
) -> Result<(), Error> {
    let old = match old_if_available {
        Some(m) => m,
        None => return Ok(()),
    };
    let difference = {
        let oldroles = &old.roles;
        let newroles = &new.roles;
        let role_set: HashSet<_> = oldroles.iter().collect();
        let difference: Vec<_> = newroles
            .iter()
            .filter(|item| !role_set.contains(item))
            .collect();
        if difference.is_empty() {
            return Ok(());
        } else {
            difference
        }
    };

    let role_id = difference.first().unwrap();
    let role_id = role_id.to_owned().to_owned();
    let pool = &user_data.pool;
    let greet_result = get_greeting(pool, &new.guild_id, &role_id).await;
    let (channel_id, greeting) = greet_result.expect("couldn't get greeting data from database");

    let role = role_id.to_role_cached(ctx.cache.clone()).unwrap();
    let greeting = greeting_replacements(ctx, greeting, role.borrow(), new, &channel_id)
        .await
        .expect("greeting replacement failed! this should never happen!");
    if channel_id.0 == 0 {
        return Ok(());
    }
    debug!("greeting attempting to be sent in channel {:?}", channel_id);

    channel_id
        .say(ctx.http.clone(), greeting)
        .await
        .expect("couldn't send the greeting. do i have perms?");
    Ok(())
}
