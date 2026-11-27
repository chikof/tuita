-- M3 schema: audit log, announcements, invites, affiliations, API tokens, passkeys.
-- Ids are uuids from Uuid::now_v7(), so ids sort by creation time.

-- `detail` is `{"text": "<a sentence>", "user": "<account id>"}`; both keys are optional.
-- `user` ties a row to an account whatever its handle is now (the account history reads it).
create table audit_log (
    id uuid primary key,
    actor_id uuid null references users on delete set null,
    action text not null,
    target text not null default '',
    detail jsonb not null default '{}',
    created_at timestamptz not null default now()
);

create index audit_log_user on audit_log ((detail->>'user'), id desc);

create table announcements (
    id uuid primary key,
    text text not null,
    tone text not null default 'info' check (tone in ('info', 'success', 'warning')),
    link text not null default '',
    active bool not null default true,
    created_by uuid null references users on delete set null,
    created_at timestamptz not null default now(),
    expires_at timestamptz null
);

create index announcements_active on announcements (active, id desc);

create table invites (
    code text primary key,
    created_by uuid null references users on delete set null,
    note text not null default '',
    max_uses int not null check (max_uses between 1 and 1000),
    uses int not null default 0,
    expires_at timestamptz null,
    revoked_at timestamptz null,
    created_at timestamptz not null default now()
);

-- users.invited_with holds the code an account signed up with.
create index users_invited_with on users (invited_with) where invited_with is not null;

create table affiliations (
    id uuid primary key,
    org_id uuid not null references users on delete cascade,
    user_id uuid not null references users on delete cascade,
    label text not null default '',
    status text not null check (status in ('pending', 'active')),
    created_at timestamptz not null default now(),
    accepted_at timestamptz null,
    unique (org_id, user_id)
);

create index affiliations_user on affiliations (user_id, status);
-- An account has at most one active affiliation.
create unique index affiliations_one_active on affiliations (user_id) where status = 'active';

-- `token_hash` is the SHA-256 of the secret; the secret is never stored. `app_id` is used by the
-- Mastodon client API in a later milestone.
create table api_tokens (
    id uuid primary key,
    user_id uuid not null references users on delete cascade,
    name text not null,
    scopes text[] not null,
    token_hash bytea not null unique,
    prefix text not null,
    app_id uuid null,
    created_at timestamptz not null default now(),
    last_used_at timestamptz null
);

create index api_tokens_user on api_tokens (user_id, id desc);

-- `passkey` is the serialised webauthn-rs `Passkey`; `cred_id` repeats its credential id for lookups.
create table passkeys (
    id uuid primary key,
    user_id uuid not null references users on delete cascade,
    cred_id bytea not null unique,
    passkey jsonb not null,
    name text not null default 'Passkey',
    created_at timestamptz not null default now(),
    last_used_at timestamptz null
);

create index passkeys_user on passkeys (user_id, created_at desc);

-- A challenge in flight. `user_id` is set for a registration and null for a sign-in. A row is
-- deleted when it is used; cleanup removes the ones that expired unused.
create table webauthn_challenges (
    id uuid primary key,
    user_id uuid null references users on delete cascade,
    state jsonb not null,
    expires_at timestamptz not null
);

create index webauthn_challenges_expires on webauthn_challenges (expires_at);

-- The id the authenticator stores as the user handle of a passkey.
alter table users add column webauthn_id uuid not null default gen_random_uuid();

-- The admin overview counts rows per day.
create index users_created on users (created_at);
create index posts_created on posts (created_at);
create index likes_created on likes (created_at);
