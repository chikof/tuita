//! The video player: a `<video>` in a frame with a dithered control bar.
//!
//! Without JS the `<video controls>` is the player. `assets/js/video.js` (`[data-island="video"]`)
//! removes `controls`, adds the class `live` to the root and drives everything else through state
//! classes on the root: `paused`, `muted`, `fs`, `pip`, `loading`, `big`, `failed`, `quiet`,
//! `can-pip`, `can-fs`. Every control is rendered here in both of its states; the CSS shows the one
//! that matches. `data-autoplay` asks the island to start playback once it takes over.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, element_id, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        icon::icon,
        icon_button::{IconButtonVariant, icon_button},
        loader::loader,
        mask::level_mask,
        menu::{MenuItem, menu},
    },
};

/// The playback speeds the speed menu offers.
const RATES: [(&str, &str); 6] = [
    ("0.5", "0.5×"),
    ("0.75", "0.75×"),
    ("1", "1×"),
    ("1.25", "1.25×"),
    ("1.5", "1.5×"),
    ("2", "2×"),
];

/// How the picture sits in the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum VideoFit {
    Cover,
    #[default]
    Contain,
}

impl VideoFit {
    const fn css(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Contain => "contain",
        }
    }
}

/// A player that fills its parent. `alt` names the video (`aria-label`; the generic "Video" when
/// empty). `looping` and `muted` are the attributes of the GIF-like clip; `autoplay` starts
/// playback once the island is live (not for a looping clip under reduced motion).
#[component]
pub(crate) async fn video_player(
    cx: &Cx,
    src: &str,
    #[default]
    #[into]
    poster: Option<&str>,
    alt: &str,
    #[default] fit: VideoFit,
    #[default] autoplay: bool,
    #[default] looping: bool,
    #[default] muted: bool,
) -> Result<impl View> {
    let generic = t!(media_video).resolve(cx).into_owned();
    let label = if alt.is_empty() {
        generic.clone()
    } else {
        alt.to_owned()
    };
    let style = format!(
        "--fit:{};--buffer-mask:{};--fill-mask:{}",
        fit.css(),
        level_mask(0.15),
        level_mask(0.5),
    );
    let menu_id = element_id(cx);
    let items: Vec<MenuItem<'_>> = RATES
        .iter()
        .map(|&(id, text)| MenuItem {
            id,
            label: text,
            ..MenuItem::default()
        })
        .collect();
    let name_video_play = t!(video_play).resolve(cx).into_owned();
    let name_video_pause = t!(video_pause).resolve(cx).into_owned();
    let name_video_unmute = t!(video_unmute).resolve(cx).into_owned();
    let name_video_mute = t!(video_mute).resolve(cx).into_owned();
    let name_video_pip = t!(video_pip).resolve(cx).into_owned();
    let name_video_exit_pip = t!(video_exit_pip).resolve(cx).into_owned();
    let name_video_fullscreen = t!(video_fullscreen).resolve(cx).into_owned();
    let name_video_exit_fullscreen = t!(video_exit_fullscreen).resolve(cx).into_owned();
    let name_video_speed = t!(video_speed).resolve(cx).into_owned();
    let speed_sr = name_video_speed.clone();
    Ok(view! {
        <div
            class="player paused"
            role="group"
            aria-label=(&generic)
            tabindex="0"
            style=(style)
            data-island="video"
            data-autoplay=(autoplay)
            data-c="video-player"
        >
            <video
                src=(src)
                poster=(poster)
                loop=(looping)
                muted=(muted)
                playsinline=""
                controls=""
                preload="metadata"
                aria-label=(&label)
            ></video>
            <span class="chip">loader(size: 24)</span>
            <button
                type="button"
                class="big wash"
                aria-label=(t!(video_play))
                data-act="toggle"
            >
                icon(name: Icon::Play, size: 32)
            </button>
            <div class="failed" role="alert">
                <p>(t!(video_error))</p>
                button(
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    attrs: attributes! { data-act="retry" },
                    (t!(retry))
                )
            </div>
            <div class="bar">
                icon_button(
                    glyph: Icon::Play,
                    label: &name_video_play,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="toggle" data-state="paused" }
                )
                icon_button(
                    glyph: Icon::Pause,
                    label: &name_video_pause,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="toggle" data-state="playing" }
                )
                <span class="time" aria-hidden="true">
                    <span data-part="now">"0:00"</span>
                    " / "
                    <span data-part="total">"0:00"</span>
                </span>
                <div
                    class="scrub off"
                    role="slider"
                    tabindex="0"
                    aria-label=(t!(video_seek))
                    aria-valuemin="0"
                    aria-valuemax="0"
                    aria-valuenow="0"
                    aria-disabled="true"
                >
                    <span class="rail" aria-hidden="true">
                        <i class="buffer"></i>
                        <i class="fill"></i>
                    </span>
                </div>
                icon_button(
                    glyph: Icon::VolumeX,
                    label: &name_video_unmute,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="mute" data-state="muted" }
                )
                icon_button(
                    glyph: Icon::Volume2,
                    label: &name_video_mute,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="mute" data-state="loud" }
                )
                <span class="vol">
                    <span class="vrail" aria-hidden="true"><i></i></span>
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.05"
                        value="1"
                        aria-label=(t!(video_volume))
                    >
                </span>
                menu(
                    id: &menu_id,
                    label: &name_video_speed,
                    items: items,
                    plain: true,
                    trigger: view! {
                        <span class="speed">
                            icon(name: Icon::SettingsCog, size: 16)
                            <span class="rate" aria-hidden="true"></span>
                            <span class="sr-only">(&speed_sr)</span>
                        </span>
                    }.into(

                    )
                )
                icon_button(
                    glyph: Icon::PictureInPicture,
                    label: &name_video_pip,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="pip" data-state="out" }
                )
                icon_button(
                    glyph: Icon::PictureInPicture,
                    label: &name_video_exit_pip,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="pip" data-state="in" }
                )
                icon_button(
                    glyph: Icon::Expand,
                    label: &name_video_fullscreen,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="fullscreen" data-state="out" }
                )
                icon_button(
                    glyph: Icon::Collapse,
                    label: &name_video_exit_fullscreen,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline,
                    attrs: attributes! { data-act="fullscreen" data-state="in" }
                )
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::{attr_values, render};

    fn sample(alt: &'static str, fit: VideoFit, autoplay: bool) -> String {
        render("en", |cx| {
            view! {
                video_player(
                    src: "/media/a.mp4",
                    poster: "/media/a.jpg",
                    alt: alt,
                    fit: fit,
                    autoplay: autoplay
                )
            }
            .boxed()
        })
    }

    #[test]
    fn the_root_is_a_focusable_named_group_for_the_island() {
        let html = sample("A cat", VideoFit::Contain, false);
        assert!(html.contains(r#"data-c="video-player""#), "{html}");
        assert!(html.contains(r#"data-island="video""#), "{html}");
        assert!(html.contains(r#"class="player paused""#), "{html}");
        assert!(html.contains(r#"role="group""#), "{html}");
        assert!(
            attr_values(&html, "aria-label").contains(&"Video".to_owned()),
            "{html}"
        );
        assert!(html.contains(r#"tabindex="0""#), "{html}");
        assert!(html.contains("--fit:contain"), "{html}");
        assert!(!html.contains("data-autoplay"), "{html}");
    }

    #[test]
    fn the_video_keeps_poster_preload_and_the_alt_as_its_name() {
        let html = sample("A cat", VideoFit::Cover, true);
        assert!(html.contains(r#"poster="/media/a.jpg""#), "{html}");
        assert!(html.contains(r#"src="/media/a.mp4""#), "{html}");
        assert!(html.contains(r#"preload="metadata""#), "{html}");
        assert!(html.contains("playsinline"), "{html}");
        assert!(html.contains(" controls"), "{html}");
        assert!(
            attr_values(&html, "aria-label").contains(&"A cat".to_owned()),
            "{html}"
        );
        assert!(html.contains("--fit:cover"), "{html}");
        assert!(html.contains("data-autoplay"), "{html}");
    }

    #[test]
    fn an_empty_alt_falls_back_to_the_generic_name() {
        let html = sample("", VideoFit::Contain, false);
        let video = &html[html.find("<video").expect("video")..html.find("</video>").expect("end")];
        assert!(video.contains(r#"aria-label="Video""#), "{video}");
    }

    #[test]
    fn every_control_is_named_in_both_of_its_states() {
        let html = sample("A cat", VideoFit::Contain, false);
        let names = attr_values(&html, "aria-label");
        for name in [
            "Play",
            "Pause",
            "Mute",
            "Unmute",
            "Volume",
            "Seek",
            "Playback speed",
            "Picture in picture",
            "Exit picture in picture",
            "Full screen",
            "Exit full screen",
        ] {
            assert!(names.contains(&name.to_owned()), "{name}: {names:?}");
        }
        assert!(html.contains(r#"role="slider""#), "{html}");
        assert!(html.contains(r#"role="alert""#), "{html}");
        assert_eq!(html.matches(r#"role="menuitem""#).count(), 6, "{html}");
    }
}
