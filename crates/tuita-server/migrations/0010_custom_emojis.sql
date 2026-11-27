-- Custom emotes: `:shortcode:` images. A local emote (`domain` null) is uploaded by staff and
-- usable by every local account; a remote one is the copy of an emote another instance sent with
-- a post or an actor, shown only on that instance's content. A shortcode is unique
-- case-insensitively per origin. The image is a `media` row, which cleanup leaves alone while an
-- emote uses it. `source_url` is the icon address a remote copy was fetched from (null for local
-- emotes); a changed address means the other instance replaced the image.
create table custom_emojis (
    id uuid primary key,
    shortcode text not null,
    domain text null,
    media_id uuid not null references media,
    category text null,
    source_url text null,
    created_by uuid null references users on delete set null,
    created_at timestamptz not null default now()
);

create unique index custom_emojis_origin_shortcode
    on custom_emojis (lower(shortcode), (coalesce(domain, '')));

create index custom_emojis_media on custom_emojis (media_id);
