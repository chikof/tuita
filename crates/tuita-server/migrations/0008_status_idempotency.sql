-- M6: `Idempotency-Key` of `POST /api/v1/statuses`. A client that retries a post with the same key
-- gets the first result back instead of a second post. A row is claimed before the post is made
-- (both result columns null), then filled in. Rows older than an hour are removed by the next
-- request that claims a key.
create table status_idempotency (
    user_id uuid not null references users on delete cascade,
    key text not null,
    post_id uuid null,
    scheduled_id uuid null,
    created_at timestamptz not null default now(),
    primary key (user_id, key)
);

create index status_idempotency_created on status_idempotency (created_at);
