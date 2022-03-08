use poise::serenity_prelude::{ChannelId, GuildId, RoleId};
use sqlx::postgres::{PgPool, PgPoolOptions};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn get_db_pool(db_connection: String) -> Result<(PgPool), Error> {
    let connection_string = &db_connection;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(connection_string)
        .await?;

    Ok(pool)
}

pub async fn set_greeting_internal(
    pool: &PgPool,
    guild_id: &GuildId,
    channel_id: ChannelId,
    role_id: RoleId,
    greeting_text: String,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO greeting_info (guild_id,channel_id,role_id,greeting,timeout)\
            VALUES ($1,$2,$3,$4,$5)\
            ON CONFLICT (guild_id) DO UPDATE \
            SET greeting = $4;",
        guild_id.0 as i64,
        channel_id.0 as i64,
        role_id.0 as i64,
        greeting_text,
        false
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_greeting(
    pool: &PgPool,
    guild_id: &GuildId,
    role_id: &RoleId,
) -> Result<(ChannelId, String), Error> {
    let cursor = sqlx::query!(
        "SELECT channel_id, greeting FROM greeting_info WHERE guild_id = $1 AND role_id = $2",
        guild_id.0 as i64,
        role_id.0 as i64
    )
    .fetch_all(pool)
    .await?;

    let mut channel_id: i64 = 0;
    let mut greeting: String = "".to_string();
    for items in cursor {
        channel_id = items.channel_id;
        greeting = items.greeting;
    }
    let channel_id = channel_id as u64;

    return Ok((ChannelId::from(channel_id), greeting));
}

pub async fn remove_greeting_internal(
    pool: &PgPool,
    guild_id: &GuildId,
    channel_id: &ChannelId,
    role_id: &RoleId,
) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM greeting_info WHERE guild_id = $1 AND channel_id = $2 AND role_id = $3",
        guild_id.0 as i64,
        channel_id.0 as i64,
        role_id.0 as i64,
    )
    .execute(pool)
    .await?;
    Ok(())
}

//NOTE: it seems ill have to re-implement removal functions for all situations that need to trigger a deletion
//      there might be a cleaner way to do this using generics, if i figure that out expect these functions to go bye-bye
pub async fn remove_greeting_by_channel(
    pool: &PgPool,
    channel_id: &ChannelId,
) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM greeting_info WHERE channel_id = $1 ",
        channel_id.0 as i64,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_greeting_by_role(pool: &PgPool, role_id: &RoleId) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM greeting_info WHERE role_id = $1",
        role_id.0 as i64,
    )
    .execute(pool)
    .await?;
    Ok(())
}
