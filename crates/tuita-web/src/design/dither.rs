//! Gallery sections: card, bento, dither-field, dither-fill, dither-text, dither-number,
//! utilities.
//!
//! The roll button, the clock and the "Set progress" style demos are plain hooks
//! (`[data-design-act="roll"]`, `[data-design-clock]`) handled by `design-parts/core.js`.

use std::time::{SystemTime, UNIX_EPOCH};

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, attributes, component, view},
};

use super::section::{row, section};
use crate::ui::{
    bento::bento,
    button::{ButtonSize, ButtonVariant, button},
    card::{CardLayout, card},
    dither_field::{FieldKind, dither_field},
    dither_fill::dither_fill,
    dither_number::dither_number,
    dither_text::dither_text,
    number_format::NumberFormat,
    tag::tag,
};

/// Heights in percent of the activity bars.
const BARS: [u32; 10] = [35, 60, 42, 85, 55, 72, 38, 95, 64, 80];

/// The levels of the `dither-fill` swatches.
const LEVELS: [f32; 9] = [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0];

/// Hour, minute and second of the day (UTC) the page was rendered; the script replaces them with
/// the browser's local time.
fn clock() -> (u64, u64, u64) {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    (seconds / 3600 % 24, seconds / 60 % 60, seconds % 60)
}

/// A captioned, square, bordered frame for a canvas.
#[component]
async fn frame(cx: &Cx, caption: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <figure data-c="design-frame">
            <div class="frame">(child)</div>
            <figcaption class="mono-tag">(caption)</figcaption>
        </figure>
    })
}

/// A captioned swatch of one dither level.
#[component]
async fn swatch(cx: &Cx, level: f32) -> Result<impl View> {
    Ok(view! {
        <figure data-c="design-frame">
            <div class="swatch">dither_fill(level: level)</div>
            <figcaption class="mono-tag">(level)</figcaption>
        </figure>
    })
}

#[component]
pub(crate) async fn dither(cx: &Cx) -> Result<impl View> {
    let (hours, minutes, seconds) = clock();
    Ok(view! {
        section(
            id: "card",
            title: "Card",
            note: "surface, 1px line, 18px radius; interactive lifts -5px with a rising wash",
            <div data-c="design-bento">
                bento(
                    card(
                        layout: CardLayout::Spread,
                        tag("static")
                        <p>"No hover motion."</p>
                    )
                    card(
                        interactive: true,
                        layout: CardLayout::Spread,
                        tag("interactive")
                        <p>"Hover or focus to lift."</p>
                    )
                    card(
                        interactive: true,
                        span: 2,
                        layout: CardLayout::Spread,
                        entrance: 0u32,
                        tag("entrance")
                        <p>
                            "Staggered entrance, 90 ms per index (landing and admin only)."
                        </p>
                    )
                )
            </div>
        )
        section(
            id: "bento",
            title: "Bento",
            note: "4 columns, 120px rows; 2 columns under 760px; the demo layout",
            <div data-c="design-bento">
                bento(
                    card(
                        interactive: true,
                        span: 2,
                        rows: 2,
                        layout: CardLayout::Spread,
                        attrs: attributes! { class="hero" },
                        dither_field(field: FieldKind::Waves, opacity: 0.5)
                        tag("01 / featured")
                        <div class="hero-copy">
                            <div class="rule"></div>
                            <h2>
                                "Every card, "
                                <span class="stroke">"two\u{a0}inks."</span>
                            </h2>
                        </div>
                        <p>
                            "A bento layout rendered in pure one-bit dither: grayscale from contrast alone, motion from intent."
                        </p>
                    )
                    card(
                        interactive: true,
                        layout: CardLayout::Spread,
                        tag("02 / signal")
                        <div>
                            <div class="num" data-design-target="">
                                dither_number(value: 97)
                                <span class="unit">"%"</span>
                            </div>
                            <div class="sub">"contrast ratio"</div>
                        </div>
                    )
                    card(
                        interactive: true,
                        layout: CardLayout::Spread,
                        tag("03 / pixels")
                        <div>
                            <div class="num">
                                "65K"
                                <span class="unit">"px"</span>
                            </div>
                            <div class="sub">"canvas budget"</div>
                        </div>
                    )
                    card(
                        interactive: true,
                        span: 2,
                        layout: CardLayout::Spread,
                        tag("04 / activity")
                        <div class="bars" aria-hidden="true">
                            for (i, height) in BARS.into_iter().enumerate() {
                                <span class="bar" style=(format!("height:{height}%"))>
                                    dither_fill(level: if i % 2 == 1 { 1.0 } else { 0.5 })
                                    <i class="cap"></i>
                                </span>
                            }
                        </div>
                    )
                    card(
                        interactive: true,
                        layout: CardLayout::Spread,
                        tag("05 / time")
                        <div class="time" data-design-clock="">
                            <span data-part="h">(format!("{hours:02}"))</span>
                            <span class="colon">":"</span>
                            <span data-part="m">(format!("{minutes:02}"))</span>
                            <span class="colon">":"</span>
                            <span data-part="s">(format!("{seconds:02}"))</span>
                        </div>
                    )
                    card(
                        interactive: true,
                        span: 2,
                        layout: CardLayout::Spread,
                        tag("06 / notes")
                        <p>
                            <strong>"Two tones, infinite texture."</strong>
                            " Dithering trades color depth for rhythm: hover any card and the pattern does the shading."
                        </p>
                    )
                )
            </div>
            row(
                button(
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    attrs: attributes! { data-design-act="roll" },
                    "Roll counter"
                )
            )
        )
        section(
            id: "dither-field",
            title: "DitherField",
            note: "canvas Bayer fields; animated ones pause off-screen and under reduced motion",
            <div class="fields" data-c="design-fields">
                frame(caption: "waves", dither_field(opacity: 1.0, cell: 5))
                frame(
                    caption: "radial",
                    dither_field(field: FieldKind::Radial, opacity: 1.0, cell: 5)
                )
                frame(
                    caption: "linear 35°",
                    dither_field(
                        field: FieldKind::Linear,
                        angle: 35.0,
                        opacity: 1.0,
                        cell: 5
                    )
                )
                frame(
                    caption: "noise",
                    dither_field(
                        field: FieldKind::Noise,
                        seed: "7",
                        opacity: 1.0,
                        cell: 5
                    )
                )
                frame(
                    caption: "ripples",
                    dither_field(
                        field: FieldKind::Ripples,
                        seed: "11",
                        opacity: 1.0,
                        cell: 5
                    )
                )
                frame(
                    caption: "orb",
                    dither_field(field: FieldKind::Orb, opacity: 1.0, cell: 5)
                )
                frame(
                    caption: "seeded ada",
                    dither_field(
                        field: FieldKind::Seeded,
                        seed: "ada",
                        opacity: 1.0,
                        cell: 5
                    )
                )
                frame(
                    caption: "seeded bob",
                    dither_field(
                        field: FieldKind::Seeded,
                        seed: "bob",
                        opacity: 1.0,
                        cell: 5
                    )
                )
                frame(
                    caption: "still",
                    dither_field(animate: false, still_time: 3.0, opacity: 1.0, cell: 5)
                )
                frame(
                    caption: "matrix 8",
                    dither_field(
                        field: FieldKind::Radial,
                        matrix: 8,
                        opacity: 1.0,
                        cell: 4
                    )
                )
                frame(
                    caption: "matrix 2",
                    dither_field(
                        field: FieldKind::Radial,
                        matrix: 2,
                        opacity: 1.0,
                        cell: 8
                    )
                )
            </div>
        )
        section(
            id: "dither-fill",
            title: "DitherFill",
            note: "levelMask(level) from 0 to 1 in Bayer order",
            <div class="levels" data-c="design-levels">
                for level in LEVELS {
                    swatch(level: level)
                }
            </div>
        )
        section(
            id: "dither-text",
            title: "DitherText",
            note: "glyphs filled with a dithered gradient; the text stays real",
            <div class="display" data-c="design-display">
                dither_text("Two inks, one bit")
            </div>
            <div class="display small" data-c="design-display">
                dither_text(from: 1.0, to: 0.7, "Quiet chrome, loud moments")
            </div>
        )
        section(
            id: "dither-number",
            title: "DitherNumber",
            note: "tabular number rolling to its target; static under reduced motion; dither fills the digits",
            row(
                <div class="big" data-c="design-big" data-design-target="">
                    dither_number(value: 97)
                </div>
                <div class="big" data-c="design-big">dither_number(value: 12480)</div>
                <div class="big" data-c="design-big">
                    dither_number(
                        value: 183204,
                        dither: true,
                        format: NumberFormat::Compact
                    )
                </div>
            )
        )
        section(
            id: "utilities",
            title: "Global dither classes",
            note: ".checker, .wash, .grain (body-level)",
            row(
                <div
                    class="box checker"
                    role="img"
                    aria-label="Checker sample"
                    data-c="design-box"
                ></div>
                <div class="box wash" tabindex="-1" data-c="design-box">
                    <span class="mono-tag">"hover for .wash"</span>
                </div>
            )
        )
    })
}
