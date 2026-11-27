//! Gallery sections: tag, badge, divider, skeleton, loader, progress-bar.
//!
//! The "Set progress" slider drives the first progress bar through the gallery script
//! (`design-parts/core.js`, `[data-design-progress]`).

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use super::section::{row, section};
use crate::ui::{
    badge::badge,
    divider::divider,
    loader::{LoaderVariant, loader},
    progress_bar::progress_bar,
    skeleton::{SkeletonShape, skeleton},
    slider::slider,
    tag::tag,
};

#[component]
pub(crate) async fn feedback(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "tag",
            title: "Tag",
            note: "mono uppercase label; pill adds a border",
            row(
                tag("01 / featured")
                tag(pill: true, "Pill tag")
                tag(pill: true, "New")
            )
        )
        section(
            id: "badge",
            title: "Badge",
            note: "count in a mono pill; dot blinks with steps(2)",
            row(
                badge(count: 3u64, label: "3 unread")
                badge(count: 42u64, label: "42 unread")
                badge(count: 120u64, label: "More than 99 unread")
                badge(dot: true, label: "New activity")
            )
        )
        section(
            id: "divider",
            title: "Divider",
            note: "dithered fade-out ends; optional centred label; vertical",
            divider()
            divider(label: "or continue with")
            row(
                label: "vertical",
                <div class="vertical" data-c="design-vertical">
                    divider(vertical: true)
                </div>
            )
        )
        section(
            id: "skeleton",
            title: "Skeleton",
            note: "0.25 checker with a dense band sweeping in steps(12)",
            row(
                skeleton(shape: SkeletonShape::Avatar, size: 48)
                <div class="lines" data-c="design-lines">
                    skeleton(shape: SkeletonShape::Line, width: "220px")
                    skeleton(shape: SkeletonShape::Line, width: "140px")
                </div>
            )
            skeleton(shape: SkeletonShape::Block, height: "96px")
        )
        section(
            id: "loader",
            title: "Loader",
            note: "ring, dots, bar",
            row(
                loader(variant: LoaderVariant::Ring, size: 32)
                loader(variant: LoaderVariant::Dots, size: 24)
                loader(variant: LoaderVariant::Bar, size: 24)
            )
        )
        section(
            id: "progress-bar",
            title: "ProgressBar",
            note: "determinate: dithered fill with a solid leading edge; indeterminate: comet",
            <div class="stack" data-c="design-narrow" data-design-progress="">
                progress_bar(value: 35.0, label: "Upload progress")
                slider(label: "Set progress", value: 35.0)
                progress_bar(value: 100.0, label: "Complete")
                progress_bar(label: "Working")
            </div>
        )
    })
}
