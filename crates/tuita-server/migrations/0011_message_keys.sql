-- End-to-end encrypted direct messages between local accounts. The browser makes the key pair; the
-- server keeps the public key, a passphrase-sealed copy of the private key it cannot open, and
-- sealed messages. A sealed row has `text = ''` and `ciphertext` set; a row without `ciphertext`
-- is plaintext (a message from a remote account, or one sent before this migration).

create table message_keys (
    user_id uuid primary key references users on delete cascade,
    public_key text not null,
    backup text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

alter table messages add column ciphertext text null;
alter table messages add constraint messages_sealed_has_no_text check (ciphertext is null or text = '');
