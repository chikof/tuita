//! Every hashed asset: stylesheets, scripts and fonts.
//!
//! The asset bundle keeps only the assets some view renders, so the root layout renders every
//! constant here. A new stylesheet or script goes into the lists below, in load order.

use topcoat::asset::{Asset, asset};

pub(crate) const TOKENS_CSS: Asset = asset!("assets/css/tokens.css");
pub(crate) const BASE_CSS: Asset = asset!("assets/css/base.css");
pub(crate) const DITHER_CSS: Asset = asset!("assets/css/dither.css");
pub(crate) const UI_CSS: Asset = asset!("assets/css/ui.css");
pub(crate) const SHELL_CSS: Asset = asset!("assets/css/shell.css");
pub(crate) const AUTH_CSS: Asset = asset!("assets/css/auth.css");
pub(crate) const POST_CSS: Asset = asset!("assets/css/post.css");
pub(crate) const MEDIA_CSS: Asset = asset!("assets/css/media.css");
pub(crate) const COMPOSE_CSS: Asset = asset!("assets/css/compose.css");
pub(crate) const FEED_CSS: Asset = asset!("assets/css/feed.css");
pub(crate) const LISTS_CSS: Asset = asset!("assets/css/lists.css");
pub(crate) const PROFILE_CSS: Asset = asset!("assets/css/profile.css");
pub(crate) const EXPLORE_CSS: Asset = asset!("assets/css/explore.css");
pub(crate) const NOTIFICATIONS_CSS: Asset = asset!("assets/css/notifications.css");
pub(crate) const MESSAGES_CSS: Asset = asset!("assets/css/messages.css");
pub(crate) const SETTINGS_CSS: Asset = asset!("assets/css/settings.css");
pub(crate) const ADMIN_CSS: Asset = asset!("assets/css/admin.css");
pub(crate) const OAUTH_CSS: Asset = asset!("assets/css/oauth.css");
pub(crate) const DOCS_CSS: Asset = asset!("assets/css/docs.css");
pub(crate) const DESIGN_CSS: Asset = asset!("assets/css/design.css");

/// Every stylesheet, in the order the layout links them.
pub(crate) const STYLESHEETS: [Asset; 20] = [
    TOKENS_CSS,
    BASE_CSS,
    DITHER_CSS,
    UI_CSS,
    SHELL_CSS,
    AUTH_CSS,
    POST_CSS,
    MEDIA_CSS,
    COMPOSE_CSS,
    FEED_CSS,
    LISTS_CSS,
    PROFILE_CSS,
    EXPLORE_CSS,
    NOTIFICATIONS_CSS,
    MESSAGES_CSS,
    SETTINGS_CSS,
    ADMIN_CSS,
    OAUTH_CSS,
    DOCS_CSS,
    DESIGN_CSS,
];

/// Applies the stored theme before the first paint: a classic script, loaded blocking in `<head>`.
pub(crate) const BOOT_JS: Asset = asset!("assets/js/boot.js");

/// An island module: its file stem (the `data-js` attribute of its `<script>`) and its asset.
pub(crate) type Script = (&'static str, Asset);

/// Every island module, in the order the layout loads them. No module imports another.
pub(crate) const SCRIPTS: [Script; 30] = [
    ("prefs", asset!("assets/js/prefs.js")),
    ("toast", asset!("assets/js/toast.js")),
    ("stream", asset!("assets/js/stream.js")),
    ("time", asset!("assets/js/time.js")),
    ("infinite", asset!("assets/js/infinite.js")),
    ("dither", asset!("assets/js/dither.js")),
    ("cropper", asset!("assets/js/cropper.js")),
    ("shell", asset!("assets/js/shell.js")),
    ("shortcuts", asset!("assets/js/shortcuts.js")),
    ("webauthn", asset!("assets/js/webauthn.js")),
    ("post", asset!("assets/js/post.js")),
    ("views", asset!("assets/js/views.js")),
    ("hovercard", asset!("assets/js/hovercard.js")),
    ("video", asset!("assets/js/video.js")),
    ("lightbox", asset!("assets/js/lightbox.js")),
    ("share", asset!("assets/js/share.js")),
    ("composer", asset!("assets/js/composer.js")),
    ("upload", asset!("assets/js/upload.js")),
    ("pull", asset!("assets/js/pull.js")),
    ("newposts", asset!("assets/js/newposts.js")),
    ("lists", asset!("assets/js/lists.js")),
    ("scheduled", asset!("assets/js/scheduled.js")),
    ("explore", asset!("assets/js/explore.js")),
    ("follow", asset!("assets/js/follow.js")),
    ("notifications", asset!("assets/js/notifications.js")),
    ("e2ee", asset!("assets/js/e2ee.js")),
    ("messages", asset!("assets/js/messages.js")),
    ("settings", asset!("assets/js/settings.js")),
    ("design", asset!("assets/js/design.js")),
    ("admin", asset!("assets/js/admin.js")),
];

pub(crate) const GEIST_FONT: Asset = asset!("assets/fonts/Geist-Variable.woff2");
pub(crate) const GEIST_MONO_FONT: Asset = asset!("assets/fonts/GeistMono-Variable.woff2");
pub(crate) const GEIST_PIXEL_FONT: Asset = asset!("assets/fonts/GeistPixel-Square.woff2");

/// The clips of the `/design` video player demo.
pub(crate) const DESIGN_SAMPLE_VIDEO: Asset = asset!("assets/media/design-sample.webm");
pub(crate) const DESIGN_BROKEN_VIDEO: Asset = asset!("assets/media/design-broken.webm");
