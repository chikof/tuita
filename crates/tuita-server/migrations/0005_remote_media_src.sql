-- The URL a remote account's avatar and banner were last copied from. A profile refresh copies
-- an image again only when the actor names another URL.
alter table users
    add column avatar_src text,
    add column banner_src text;
