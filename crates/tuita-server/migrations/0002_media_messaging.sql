-- M2 schema: polls, edit history, scheduled posts, direct messages, lists, view counts, link
-- cards and reports. Ids are uuids from Uuid::now_v7(); counters change with their rows.

create table polls (
    post_id uuid primary key references posts on delete cascade,
    options text[] not null,
    multiple bool not null,
    ends_at timestamptz not null,
    counts int[] not null,
    -- Accounts that voted at least once. A multiple-choice voter adds to several `counts`.
    voters int not null default 0
);

create table poll_votes (
    post_id uuid not null references polls on delete cascade,
    user_id uuid not null references users on delete cascade,
    choice smallint not null,
    created_at timestamptz not null default now(),
    primary key (post_id, user_id, choice)
);

create table post_edits (
    id uuid primary key,
    post_id uuid not null references posts on delete cascade,
    text text not null,
    cw text not null default '',
    edited_at timestamptz not null default now()
);

create index post_edits_post on post_edits (post_id, id desc);

create table scheduled_posts (
    id uuid primary key,
    author_id uuid not null references users on delete cascade,
    payload jsonb not null,
    publish_at timestamptz not null,
    status text not null default 'pending' check (status in ('pending', 'published', 'failed')),
    post_id uuid null references posts on delete set null,
    error text null,
    created_at timestamptz not null default now()
);

create index scheduled_posts_due on scheduled_posts (status, publish_at);
create index scheduled_posts_author on scheduled_posts (author_id, publish_at);

create table messages (
    id uuid primary key,
    sender_id uuid not null references users on delete cascade,
    recipient_id uuid not null references users on delete cascade,
    text text not null,
    read_at timestamptz null,
    deleted_at timestamptz null,
    created_at timestamptz not null default now()
);

create index messages_pair on messages (least(sender_id, recipient_id), greatest(sender_id, recipient_id), id desc);
create index messages_unread on messages (recipient_id, read_at);

create table lists (
    id uuid primary key,
    owner_id uuid not null references users on delete cascade,
    name text not null,
    description text not null default '',
    private bool not null default false,
    created_at timestamptz not null default now()
);

create index lists_owner on lists (owner_id, id desc);

create table list_members (
    list_id uuid not null references lists on delete cascade,
    user_id uuid not null references users on delete cascade,
    created_at timestamptz not null default now(),
    primary key (list_id, user_id)
);

create index list_members_user on list_members (user_id);

-- One row per viewer, post and day. `viewer` is `u<user id>` or `a<salted ip hash>`.
create table post_views (
    post_id uuid not null references posts on delete cascade,
    viewer text not null,
    day date not null,
    primary key (post_id, viewer, day)
);

create index post_views_day on post_views (day);

create table link_cards (
    url text primary key,
    title text not null default '',
    description text not null default '',
    site text not null default '',
    image_id uuid null references media on delete set null,
    large bool not null default false,
    ok bool not null default false,
    fetched_at timestamptz not null default now()
);

create table reports (
    id uuid primary key,
    reporter_id uuid not null references users on delete cascade,
    target_id uuid not null references users on delete cascade,
    post_id uuid null references posts on delete set null,
    reason text not null,
    note text not null default '',
    status text not null default 'open',
    resolution text null,
    resolved_by uuid null references users on delete set null,
    created_at timestamptz not null default now(),
    resolved_at timestamptz null
);

create index reports_open on reports (status, id desc);
create index reports_reporter on reports (reporter_id, target_id);
