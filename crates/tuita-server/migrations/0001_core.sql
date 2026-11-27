-- M1 core schema: accounts, sessions, posts, the social graph and notifications.
-- Every id is a uuid generated in Rust with Uuid::now_v7(), so ids sort by creation time.
-- Counters are updated in the same transaction as the row they count.

create extension if not exists pg_trgm;

create table instance_settings (
    key text primary key,
    value jsonb not null
);

-- users.avatar_id / banner_id / pinned_post_id and media.post_id are added at the end.
create table users (
    id uuid primary key,
    handle text not null,
    domain text null,
    display_name text not null default '',
    bio text not null default '',
    location text not null default '',
    website text not null default '',
    pronouns text not null default '',
    avatar_id uuid null,
    banner_id uuid null,
    avatar_shape text not null default 'square'
        check (avatar_shape in ('square', 'circle', 'squircle', 'octagon')),
    ink text not null default 'bone'
        check (ink in ('bone', 'phosphor', 'amber', 'cyan', 'rose')),
    flags text[] not null default '{}',
    password_hash text null,
    role text not null default 'user' check (role in ('user', 'moderator', 'admin')),
    verified bool not null default false,
    suspended_at timestamptz null,
    deleted_at timestamptz null,
    is_org bool default false,
    is_bot bool default false,
    bot_owner_id uuid null references users on delete set null,
    dm_policy text not null default 'everyone' check (dm_policy in ('everyone', 'following', 'none')),
    discoverable bool default true,
    show_client bool default true,
    muted_words text[] default '{}',
    notif_prefs jsonb default '{}',
    language text null,
    invited_with text null,
    pinned_post_id uuid null,
    followers_count int not null default 0,
    following_count int not null default 0,
    posts_count int not null default 0,
    created_at timestamptz not null default now()
);

create unique index users_handle on users (lower(handle), (coalesce(domain, '')));
create index users_search on users using gin ((handle || ' ' || display_name) gin_trgm_ops);

create table media (
    id uuid primary key,
    owner_id uuid not null references users on delete cascade,
    kind text not null check (kind in ('image', 'video')),
    mime text not null,
    path text not null,
    bytes bigint not null,
    width int,
    height int,
    alt text not null default '',
    poster_id uuid references media on delete set null,
    post_id uuid null,
    created_at timestamptz not null default now()
);

create table sessions (
    id uuid primary key,
    user_id uuid not null references users on delete cascade,
    token_hash bytea not null unique,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null,
    last_used_at timestamptz not null default now(),
    user_agent text not null default '',
    ip inet null,
    country text null,
    client text null
);

create index sessions_user on sessions (user_id);

create table posts (
    id uuid primary key,
    author_id uuid not null references users on delete cascade,
    text text not null default '',
    cw text not null default '',
    sensitive bool not null default false,
    reply_to_id uuid null references posts on delete set null,
    root_id uuid null,
    quote_of_id uuid null references posts on delete set null,
    reply_policy text not null default 'everyone'
        check (reply_policy in ('everyone', 'following', 'mentioned')),
    client text null,
    edited_at timestamptz null,
    deleted_at timestamptz null,
    hidden_reply bool default false,
    views bigint default 0,
    card_url text null,
    like_count int not null default 0,
    repost_count int not null default 0,
    reply_count int not null default 0,
    quote_count int not null default 0,
    search tsvector generated always as (to_tsvector('simple', cw || ' ' || text)) stored,
    created_at timestamptz not null default now()
);

create index posts_author on posts (author_id, id desc);
create index posts_reply_to on posts (reply_to_id);
create index posts_quote_of on posts (quote_of_id);
create index posts_root on posts (root_id);
create index posts_search on posts using gin (search);

alter table users add constraint users_avatar_fk foreign key (avatar_id) references media on delete set null;
alter table users add constraint users_banner_fk foreign key (banner_id) references media on delete set null;
alter table users add constraint users_pinned_post_fk foreign key (pinned_post_id) references posts on delete set null;
alter table media add constraint media_post_fk foreign key (post_id) references posts on delete set null;

create table post_media (
    post_id uuid references posts on delete cascade,
    media_id uuid references media on delete cascade,
    position smallint not null,
    primary key (post_id, position)
);

create table post_tags (
    post_id uuid references posts on delete cascade,
    tag text not null,
    primary key (post_id, tag)
);

create index post_tags_tag on post_tags (tag, post_id desc);

create table mentions (
    post_id uuid not null references posts on delete cascade,
    user_id uuid not null references users on delete cascade,
    primary key (post_id, user_id)
);

create table likes (
    user_id uuid not null references users on delete cascade,
    post_id uuid not null references posts on delete cascade,
    created_at timestamptz not null default now(),
    primary key (user_id, post_id)
);

create index likes_post on likes (post_id);

create table reposts (
    id uuid primary key,
    user_id uuid not null references users on delete cascade,
    post_id uuid not null references posts on delete cascade,
    created_at timestamptz not null default now(),
    unique (user_id, post_id)
);

create table bookmarks (
    user_id uuid not null references users on delete cascade,
    post_id uuid not null references posts on delete cascade,
    created_at timestamptz not null default now(),
    primary key (user_id, post_id)
);

create table follows (
    follower_id uuid not null references users on delete cascade,
    followee_id uuid not null references users on delete cascade,
    state text not null default 'accepted' check (state in ('accepted', 'pending')),
    created_at timestamptz not null default now(),
    primary key (follower_id, followee_id)
);

create index follows_followee on follows (followee_id);

create table blocks (
    user_id uuid not null references users on delete cascade,
    target_id uuid not null references users on delete cascade,
    created_at timestamptz not null default now(),
    primary key (user_id, target_id)
);

create table mutes (
    user_id uuid not null references users on delete cascade,
    target_id uuid not null references users on delete cascade,
    created_at timestamptz not null default now(),
    primary key (user_id, target_id)
);

create table notifications (
    id uuid primary key,
    user_id uuid not null references users on delete cascade,
    actor_id uuid not null references users on delete cascade,
    kind text not null check (kind in
        ('like', 'repost', 'reply', 'mention', 'quote', 'follow', 'follow_request', 'poll', 'affiliation')),
    post_id uuid null references posts on delete cascade,
    read_at timestamptz null,
    created_at timestamptz not null default now()
);

create index notifications_user on notifications (user_id, id desc);

create table feed_snapshots (
    user_id uuid primary key references users on delete cascade,
    ids uuid[] not null,
    created_at timestamptz not null default now()
);
