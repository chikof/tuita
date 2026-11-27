//! `/settings/profile`.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, procedure, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AvatarShape as ShapeDto, Ink, MeDto, PatchMeRequest};

use crate::{
    app::FLASH_COOKIE,
    cookies,
    l10n::t,
    session,
    settings::{
        actions::{Done, patch_me},
        card::setting_card,
        image_field::{ImageKind, image_field},
        ink_picker::ink_picker,
        page::settings_page,
        shape_picker::shape_picker,
    },
    ui::{
        avatar::{AvatarShape, avatar},
        button::button,
        text_area::text_area,
        text_field::text_field,
    },
};

/// The editable fields of the profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProfileDraft {
    pub(crate) display_name: String,
    pub(crate) bio: String,
    pub(crate) location: String,
    pub(crate) website: String,
    pub(crate) avatar_shape: ShapeDto,
    pub(crate) ink: Ink,
}

/// What the account holds now.
pub(crate) fn draft_of(me: &MeDto) -> ProfileDraft {
    ProfileDraft {
        display_name: me.user.display_name.clone(),
        bio: me.user.bio.clone(),
        location: me.user.location.clone(),
        website: me.user.website.clone(),
        avatar_shape: me.user.avatar_shape,
        ink: me.user.ink,
    }
}

/// The `PATCH /me` body for what differs from the saved profile; empty when nothing changed.
/// Text is trimmed, so an empty website clears it.
pub(crate) fn profile_changes(me: &MeDto, draft: &ProfileDraft) -> PatchMeRequest {
    let saved = draft_of(me);
    let changed = |next: &str, old: &str| {
        let next = next.trim();
        (next != old).then(|| next.to_owned())
    };
    PatchMeRequest {
        display_name: changed(&draft.display_name, &saved.display_name),
        bio: changed(&draft.bio, &saved.bio),
        location: changed(&draft.location, &saved.location),
        website: changed(&draft.website, &saved.website),
        avatar_shape: (draft.avatar_shape != saved.avatar_shape).then_some(draft.avatar_shape),
        ink: (draft.ink != saved.ink).then_some(draft.ink),
        ..PatchMeRequest::default()
    }
}

/// Saves the profile form. The page reloads afterwards so the whole shell shows the new name, and
/// the "Profile updated" toast travels as a flash message.
#[procedure]
pub(crate) async fn save_profile(
    cx: &Cx,
    display_name: String,
    bio: String,
    location: String,
    website: String,
    shape: String,
    ink: String,
) -> Done<bool> {
    let Some(me) = session::me(cx).await? else {
        return Ok(Err("Not signed in".to_owned()));
    };
    let (Some(avatar_shape), Some(ink)) = (ShapeDto::parse(&shape), Ink::parse(&ink)) else {
        return Ok(Err("Unknown choice".to_owned()));
    };
    let draft = ProfileDraft {
        display_name,
        bio,
        location,
        website,
        avatar_shape,
        ink,
    };
    let changes = profile_changes(me, &draft);
    if changes == PatchMeRequest::default() {
        return Ok(Ok(true));
    }
    let done = patch_me(cx, &changes).await;
    if done.is_ok() {
        cookies::set(cx, FLASH_COOKIE, "settings_profile_saved", None);
    }
    Ok(done.map(|_| true))
}

/// The pictures, a live preview and the details form.
#[component]
pub(crate) async fn profile(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let user = &me.user;
    let title = t!(settings_profile).resolve(cx).into_owned();
    let name = signal(cx, || user.display_name.clone());
    let bio = signal(cx, || user.bio.clone());
    let location = signal(cx, || user.location.clone());
    let website = signal(cx, || user.website.clone());
    let shape = signal(cx, || user.avatar_shape.as_str().to_owned());
    let ink = signal(cx, || user.ink.as_str().to_owned());
    let failure = signal(cx, String::new);
    let busy = signal(cx, || false);
    let saved_name = user.display_name.clone();
    let saved_bio = user.bio.clone();
    let saved_location = user.location.clone();
    let saved_website = user.website.clone();
    let saved_shape = user.avatar_shape.as_str().to_owned();
    let saved_ink = user.ink.as_str().to_owned();
    let preview_alt = t!(settings_profile_preview_alt).resolve(cx).into_owned();
    let shapes = [
        ("square", AvatarShape::Square),
        ("circle", AvatarShape::Circle),
        ("squircle", AvatarShape::Squircle),
        ("octagon", AvatarShape::Octagon),
    ];
    let name_label = t!(settings_profile_name).resolve(cx).into_owned();
    let bio_label = t!(settings_profile_bio).resolve(cx).into_owned();
    let location_label = t!(settings_profile_location).resolve(cx).into_owned();
    let website_label = t!(settings_profile_website).resolve(cx).into_owned();
    let shape_label = t!(settings_profile_shape).resolve(cx).into_owned();
    let ink_label = t!(settings_profile_ink).resolve(cx).into_owned();
    let fallback_name = user.display_name.clone();
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: t!(image_field_banner).resolve(cx).into_owned(),
                image_field(kind: ImageKind::Banner, user: user)
            )
            setting_card(
                title: t!(image_field_avatar).resolve(cx).into_owned(),
                image_field(kind: ImageKind::Avatar, user: user)
            )
            setting_card(
                title: t!(settings_profile_preview).resolve(cx).into_owned(),
                <div class="preview" data-c="profile-preview" :data-ink=$(ink.get())>
                    for (id, kind) in shapes {
                        <span :hidden=$(shape.get() != id)>
                            avatar(
                                seed: &user.handle,
                                src: user.avatar_url.as_deref(),
                                alt: &preview_alt,
                                shape: kind,
                                size: 64
                            )
                        </span>
                    }
                    <div class="who">
                        <strong class="name">
                            $(if name.get().trim().is_empty() {
                                fallback_name
                            } else {
                                name.get().trim().to_owned()
                            })
                        </strong>
                        <span class="handle">
                            "@"
                            (&me.federated_handle)
                        </span>
                    </div>
                </div>
            )
            <form
                data-c="profile-form"
                novalidate=(true)
                @submit=$(async |e: Event| {
                    e.prevent_default();
                    busy.set(true);
                    failure.set("".to_owned());
                    let done = save_profile(
                        name.get(),
                        bio.get(),
                        location.get(),
                        website.get(),
                        shape.get(),
                        ink.get(),
                    ).await;
                    if done.is_ok() {
                        raw!("location.reload()")
                    } else {
                        failure.set(done.unwrap_err());
                        busy.set(false);
                        false
                    }
                })
            >
                setting_card(
                    title: t!(settings_profile_details).resolve(cx).into_owned(),
                    text_field(
                        label: &name_label,
                        attrs: attributes! {
                            name="displayName"
                            autocomplete="name"
                            maxlength="50"
                            required=(true)
                            :value=$(name.get())
                            @input=$(|e: Event| name.set(e.target.value))
                        }
                    )
                    <div class="bio">
                        text_area(
                            label: &bio_label,
                            attrs: attributes! {
                                name="bio"
                                maxlength="160"
                                :value=$(bio.get())
                                @input=$(|e: Event| bio.set(e.target.value))
                            }
                        )
                        <div class="counter" aria-hidden="true">
                            $(bio.get().len())
                            "/160"
                        </div>
                    </div>
                    text_field(
                        label: &location_label,
                        attrs: attributes! {
                            name="location"
                            maxlength="30"
                            :value=$(location.get())
                            @input=$(|e: Event| location.set(e.target.value))
                        }
                    )
                    text_field(
                        label: &website_label,
                        attrs: attributes! {
                            name="website"
                            type="url"
                            inputmode="url"
                            maxlength="100"
                            :value=$(website.get())
                            @input=$(|e: Event| website.set(e.target.value))
                        }
                    )
                    shape_picker(
                        value: &shape,
                        seed: &user.handle,
                        src: user.avatar_url.as_deref(),
                        label: &shape_label
                    )
                    ink_picker(value: &ink, label: &ink_label)
                    <p class="failure" role="alert" :hidden=$(failure.get().is_empty())>
                        $(failure.get())
                    </p>
                    <div class="end">
                        button(
                            submit: true,
                            attrs: attributes! {
                                :disabled=$(if busy.get() {
                                    true
                                } else if name.get().trim() != saved_name {
                                    false
                                } else if bio.get().trim() != saved_bio {
                                    false
                                } else if location.get().trim() != saved_location {
                                    false
                                } else if website.get().trim() != saved_website {
                                    false
                                } else if shape.get() != saved_shape {
                                    false
                                } else {
                                    ink.get() == saved_ink
                                })
                            },
                            (t!(settings_profile_save))
                        )
                    </div>
                )
            </form>
        )
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn me() -> MeDto {
        serde_json::from_value(serde_json::json!({
            "user": {
                "id": "00000000-0000-0000-0000-000000000001",
                "handle": "ada", "domain": null, "displayName": "Ada", "bio": "", "location": "",
                "website": "https://ada.example", "avatarUrl": null, "bannerUrl": null,
                "avatarShape": "circle", "ink": "bone", "verified": false, "staff": null,
                "org": false, "bot": false, "botOwner": null, "affiliation": null,
                "pinnedPostId": null, "createdAt": "2026-01-02T03:04:05Z",
                "counts": {"followers": 0, "following": 0, "posts": 0},
                "relation": {"following": false, "followsYou": false, "blocked": false,
                    "blocksYou": false, "muted": false, "followPending": false},
                "suspended": false, "url": null, "locked": false, "emojis": []
            },
            "role": "user",
            "settings": {
                "dmPolicy": "everyone", "discoverable": true, "language": null, "showClient": true,
                "notifPrefs": {"like": true, "repost": true, "follow": true, "reply": true,
                    "mention": true, "quote": true},
                "mutedWords": []
            },
            "federatedHandle": "ada@tuita.test"
        }))
        .expect("a valid MeDto")
    }

    #[test]
    fn an_untouched_draft_changes_nothing() {
        let me = me();
        assert_eq!(
            profile_changes(&me, &draft_of(&me)),
            PatchMeRequest::default()
        );
    }

    #[test]
    fn only_changed_fields_are_sent_and_an_empty_website_clears_it() {
        let me = me();
        let draft = ProfileDraft {
            display_name: " Ada L ".to_owned(),
            website: String::new(),
            ink: Ink::Rose,
            ..draft_of(&me)
        };
        assert_eq!(
            profile_changes(&me, &draft),
            PatchMeRequest {
                display_name: Some("Ada L".to_owned()),
                website: Some(String::new()),
                ink: Some(Ink::Rose),
                ..PatchMeRequest::default()
            }
        );
    }
}
