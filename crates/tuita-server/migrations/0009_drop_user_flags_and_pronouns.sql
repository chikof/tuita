-- Pride flags and pronouns are no longer profile fields.
alter table users drop column flags, drop column pronouns;
