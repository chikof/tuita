-- M4 schema: ActivityPub federation. Remote accounts are `users` rows with a `domain`; remote posts
-- are `posts` rows with an `ap_id`. Local accounts and posts keep a null `ap_id`: their ids are
-- computed from PUBLIC_URL and the handle or post id.

alter table users
    add column ap_id text unique,
    add column inbox_url text,
    add column shared_inbox_url text,
    add column outbox_url text,
    add column followers_url text,
    add column featured_url text,
    add column url text,
    add column public_key_pem text,
    -- PKCS#8 PEM of a local account's key. Generated at sign-up, lazily for older accounts.
    add column private_key_pem text,
    add column fetched_at timestamptz,
    add column manually_approves bool default false;

create index users_domain on users (domain) where domain is not null;

alter table posts
    add column ap_id text unique,
    add column url text,
    add column visibility text not null default 'public'
        check (visibility in ('public', 'unlisted', 'followers', 'direct'));

-- Timelines, search and tag lists read public posts only.
create index posts_public on posts (id desc) where visibility = 'public' and deleted_at is null;

-- The id of the Follow activity, so an Accept or an Undo can name it.
alter table follows add column ap_id text;
alter table messages add column ap_id text unique;
alter table reposts add column ap_id text unique;

-- `silence` hides an instance's accounts from public timelines and search; `suspend` drops
-- everything to and from it.
create table domain_blocks (
    domain text primary key,
    severity text not null check (severity in ('silence', 'suspend')),
    reason text not null default '',
    created_at timestamptz not null default now()
);

-- One row per activity per inbox. `signer_id` is the local account whose key signs the delivery;
-- null means the instance actor. `domain` is the inbox's host, for the queue view and for removing
-- the rows of a blocked instance.
create table ap_deliveries (
    id uuid primary key,
    inbox_url text not null,
    activity jsonb not null,
    signer_id uuid null references users on delete set null,
    attempts int not null default 0,
    next_attempt_at timestamptz not null default now(),
    last_error text null,
    created_at timestamptz not null default now(),
    domain text generated always as (lower(substring(inbox_url from '^https?://([^/]+)'))) stored
);

create index ap_deliveries_due on ap_deliveries (next_attempt_at);
create index ap_deliveries_domain on ap_deliveries (domain);

-- Activities already handled, so a replayed delivery changes nothing. `cleanup` prunes old rows.
create table ap_seen (
    activity_id text primary key,
    received_at timestamptz not null default now()
);

create index ap_seen_received on ap_seen (received_at);

-- The instance actor's key pair lives in instance_settings key `instance_actor`
-- ({"publicPem": "...", "privatePem": "..."}); it is created on first use.

-- The system account that files reports received from other instances (`Flag`). It has no
-- password, so nobody signs in as it, and is not discoverable. `tuita` is a reserved handle, and
-- the server serves it nowhere. An existing local user with that handle would silently become
-- the report filer, so the migration refuses to run until that user is renamed or deleted.
do $$
begin
    if exists (select 1 from users where lower(handle) = 'tuita' and domain is null) then
        raise exception 'a local user already has the reserved handle "tuita": rename or delete it, then run the migration again';
    end if;
end
$$;

insert into users (id, handle, display_name, role, discoverable)
values (gen_random_uuid(), 'tuita', 'tuita', 'user', false);
