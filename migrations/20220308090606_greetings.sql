-- Add migration script here
create TABLE public.greeting_info
(
    guild_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    role_id bigint NOT NULL,
    greeting text NOT NULL,
    timeout bool NOT NULL,
    -- time out duration?
    PRIMARY KEY (role_id)

);
alter table public.greeting_info
    OWNER to emerald;