-- Remote note ids that were deleted. A `Delete` can overtake the `Create` it follows (two
-- deliveries, two retries, a backfill), and a note that is not stored when the `Delete` comes
-- leaves nothing to mark deleted. Storing a note checks this table first, so the late `Create`
-- does not bring the note back. `cleanup` prunes rows after 30 days, longer than any sender
-- retries a delivery.
create table ap_tombstones (
    ap_id text primary key,
    deleted_at timestamptz not null default now()
);

create index ap_tombstones_deleted on ap_tombstones (deleted_at);
