-- M6: OAuth2 for Mastodon-compatible clients.
--
-- `oauth_apps` are the clients that registered themselves (`POST /api/v1/apps`). Only the SHA-256
-- of a `client_secret` is stored. `redirect_uris` and `scopes` are kept as registered; the
-- authorization endpoint compares a redirect URI by exact string equality.
create table oauth_apps (
    id uuid primary key,
    client_id text not null unique,
    client_secret_hash bytea not null,
    name text not null,
    redirect_uris text[] not null,
    scopes text[] not null,
    website text null,
    created_at timestamptz not null default now()
);

-- An authorization code in flight. Only its SHA-256 is stored. `used_at` is set in the
-- transaction that issues the token, and `token_id` names that token: presenting a used code
-- again deletes the token (RFC 6749 4.1.2). Rows live a day past their expiry so that a replay
-- of an expired but used code is still recognised; `cleanup` removes them then.
create table oauth_codes (
    code_hash bytea primary key,
    app_id uuid not null references oauth_apps on delete cascade,
    user_id uuid not null references users on delete cascade,
    scopes text[] not null,
    redirect_uri text not null,
    code_challenge text null,
    expires_at timestamptz not null,
    used_at timestamptz null,
    token_id uuid null
);

create index oauth_codes_expires on oauth_codes (expires_at);

-- OAuth tokens are `api_tokens` rows with `app_id` set. A `client_credentials` token belongs to
-- the app alone, so `user_id` may now be null; the bearer lookup joins `users`, so such a token
-- never authenticates a user route.
alter table api_tokens alter column user_id drop not null;
alter table api_tokens
    add constraint api_tokens_app_fk foreign key (app_id) references oauth_apps on delete cascade;
alter table api_tokens
    add constraint api_tokens_owner check (user_id is not null or app_id is not null);
alter table oauth_codes
    add constraint oauth_codes_token_fk foreign key (token_id) references api_tokens on delete set null;

create index api_tokens_app on api_tokens (app_id, id desc) where app_id is not null;
