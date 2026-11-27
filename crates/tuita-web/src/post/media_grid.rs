//! A post's pictures and videos.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::{MediaDto, MediaKind, PostDto};

use crate::{
    icons::Icon,
    l10n::{strip_isolates, t},
    post::{
        carousel::{COLUMN_FALLBACK, aspect_of, carousel_layout, scrolls},
        sensitive_cover::sensitive_cover,
        video_player::{VideoFit, video_player},
    },
    ui::{
        icon::icon,
        icon_button::{IconButtonVariant, icon_button},
        skeleton::{SkeletonShape, skeleton},
    },
};

/// One media item: a video player, or a picture that opens the lightbox.
///
/// A picture button carries `data-act="lightbox"`, `data-index` and, when the grid knows its
/// post, `data-post-id`; `lightbox.js` fetches `/_tuita/lightbox/<id>?index=<n>` for it.
#[component]
async fn cell(
    cx: &Cx,
    item: &MediaDto,
    index: usize,
    count: usize,
    post_id: Option<&str>,
) -> Result<impl View> {
    let video = item.kind == MediaKind::Video;
    let video_alt = if item.alt.is_empty() {
        t!(media_video).resolve(cx).into_owned()
    } else {
        item.alt.clone()
    };
    let label = if item.alt.is_empty() {
        strip_isolates(&t!(media_open, index: index + 1, count: count).resolve(cx))
    } else {
        item.alt.clone()
    };
    Ok(view! {
        if video {
            video_player(
                src: &item.url,
                poster: item.poster_url.as_deref(),
                alt: &video_alt,
                fit: VideoFit::Contain
            )
        } else {
            <span class="skeleton">
                skeleton(shape: SkeletonShape::Block, width: "100%", height: "100%")
            </span>
            <button
                type="button"
                class="open"
                aria-label=(&label)
                data-act="lightbox"
                data-index=(index.to_string())
                data-post-id=(post_id)
            >
                <img
                    src=(&item.url)
                    alt=(&item.alt)
                    width=(item.width)
                    height=(item.height)
                    loading="lazy"
                    decoding="async"
                >
            </button>
            if !item.alt.is_empty() {
                <span class="alt" aria-hidden="true">
                    icon(name: Icon::InfoBox, size: 16)
                    (t!(media_alt))
                </span>
            }
        }
    })
}

#[component]
async fn grid(cx: &Cx, media: &[MediaDto], post_id: Option<&str>) -> Result<impl View> {
    let shown = &media[..media.len().min(4)];
    let count = shown.len();
    let first = &media[0];
    let label = strip_isolates(&t!(media_label, count: count).resolve(cx));
    let aspects: Vec<f64> = shown
        .iter()
        .map(|item| aspect_of(item.width, item.height))
        .collect();
    let layout = carousel_layout(&aspects, COLUMN_FALLBACK);
    let overflow = count > 1 && scrolls(&layout.widths, COLUMN_FALLBACK);
    let single = if count == 1
        && first.width.is_some_and(|w| w > 0)
        && first.height.is_some_and(|h| h > 0)
    {
        aspect_of(first.width, first.height).clamp(4.0 / 5.0, 16.0 / 9.0)
    } else {
        16.0 / 9.0
    };
    let cells: Vec<_> = shown
        .iter()
        .enumerate()
        .map(|(index, item)| (index, item, layout.widths[index], aspects[index]))
        .collect();
    let pager = strip_isolates(&t!(media_pager, index: 1, count: count).resolve(cx));
    let prev = t!(lightbox_prev).resolve(cx).into_owned();
    let next = t!(lightbox_next).resolve(cx).into_owned();
    Ok(view! {
        if count == 1 {
            <div
                class="frame"
                style=(format!("aspect-ratio:{single}"))
                role="group"
                aria-label=(&label)
                data-c="media-grid"
            >
                <div class="cell fill">
                    cell(item: first, index: 0, count: count, post_id: post_id)
                </div>
            </div>
        } else {
            <div class="carousel" data-island="media-grid" data-c="media-grid">
                <div
                    class="track"
                    role="group"
                    tabindex="0"
                    aria-label=(&label)
                    style=(format!("height:{}px", layout.height))
                >
                    #[key(index)]
                    for (index, item, width, aspect) in cells {
                        <div
                            class="cell"
                            style=(format!("width:{width}px"))
                            data-aspect=(aspect.to_string())
                        >
                            cell(
                                item: item,
                                index: index,
                                count: count,
                                post_id: post_id
                            )
                        </div>
                    }
                </div>
                <span class="pager" aria-hidden="true" hidden=(!overflow)>(pager)</span>
                <span class="nav prev" hidden="">
                    icon_button(
                        glyph: Icon::ChevronLeft,
                        label: &prev,
                        variant: IconButtonVariant::Outline,
                        attrs: attributes! { data-act="media-prev" }
                    )
                </span>
                <span class="nav next" hidden=(!overflow)>
                    icon_button(
                        glyph: Icon::ChevronRight,
                        label: &next,
                        variant: IconButtonVariant::Outline,
                        attrs: attributes! { data-act="media-next" }
                    )
                </span>
            </div>
        }
    })
}

/// A post's media: one picture at its own shape, or a scrolling strip of up to four. `sensitive`
/// puts it behind a cover. `post` names the post the lightbox opens for.
///
/// `lightbox.js` (`[data-island="media-grid"]`) measures the strip, applies the same layout as
/// `carousel_layout` and keeps the pager and arrows current; the arrow keys scroll a focused
/// strip.
#[component]
pub(crate) async fn media_grid(
    cx: &Cx,
    media: &[MediaDto],
    #[default] sensitive: bool,
    #[default] post: Option<&PostDto>,
) -> Result<impl View> {
    let id = post.map(|post| post.id.to_string());
    Ok(view! {
        if !media.is_empty() {
            if sensitive {
                sensitive_cover(grid(media: media, post_id: id.as_deref()))
            } else {
                grid(media: media, post_id: id.as_deref())
            }
        }
    })
}
