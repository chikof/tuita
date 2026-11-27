//! Known-answer vectors of the browser islands.
//!
//! The page loads every island as `<script type="module" data-js="<name>" src=...>`. A test opens
//! `/`, imports that module by its URL and hands it to the vector's `run(mod)` from
//! `js/units/<name>.test.js`, which throws on the first failed assertion. Adding an island's
//! vectors: put the file there and append one `unit!` line below.

use tuita_e2e::{Ctx, Result, helpers::run_js_unit};

async fn run_unit(name: &str, vector: &str) -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    // The vector is a module loaded from a blob: URL, which `script-src 'self'` refuses.
    page.bypass_csp().await?;
    page.goto("/").await?;
    run_js_unit(&page, name, vector).await
}

macro_rules! unit {
    ($name:ident) => {
        unit!($name, stringify!($name));
    };
    ($name:ident, $module:expr) => {
        #[tokio::test]
        async fn $name() -> Result<()> {
            run_unit(
                $module,
                include_str!(concat!("../../js/units/", stringify!($name), ".test.js")),
            )
            .await
        }
    };
}

unit!(stream);
unit!(time);
unit!(dither);
unit!(crop, "cropper");
unit!(shell);
unit!(shortcuts);
unit!(webauthn);
unit!(compose, "composer");
unit!(schedule, "composer");
unit!(emojis, "composer");
unit!(drafts, "composer");
unit!(upload);
unit!(media_tray, "upload");
unit!(views);
unit!(hovercard);
unit!(share);
unit!(video);
unit!(lightbox);
unit!(post);
unit!(e2ee);
unit!(e2ee_store, "e2ee");
unit!(e2ee_lines, "e2ee");
unit!(e2ee_conversation, "e2ee");
unit!(e2ee_send, "e2ee");
unit!(e2ee_keys, "e2ee");
unit!(messages);
unit!(settings);
unit!(pull);
unit!(newposts);
unit!(lists);
unit!(scheduled);
unit!(admin);
unit!(follow);
unit!(explore);
unit!(notifications);
