//! Decoding, orientation, resizing and metadata-free re-encoding of uploaded images.
//!
//! Everything here is synchronous CPU work; callers run it in `spawn_blocking`. The output is
//! built from decoded pixels only, so EXIF, ICC profiles and comments never reach the disk.

use std::io::Cursor;

use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::imageops::{self, FilterType};
use image::metadata::Orientation;
use image::{
    AnimationDecoder as _, DynamicImage, ExtendedColorType, Frame, ImageDecoder as _, ImageError,
    ImageFormat, ImageReader, Limits,
};

use crate::dto::CropRect;

/// Longest edge of a stored post image.
pub(crate) const MAX_EDGE: u32 = 2048;
/// Longest edge of a stored custom emote.
const EMOTE_EDGE: u32 = 128;
/// Stored avatar size.
const AVATAR: (u32, u32) = (400, 400);
/// Stored banner size.
const BANNER: (u32, u32) = (1500, 500);
/// JPEG quality of re-encoded photos.
const JPEG_QUALITY: u8 = 82;
/// Widest or tallest image the decoders accept.
const MAX_DIMENSION: u32 = 16384;
/// Most memory a decoder may allocate: enough for a 48 MP photo, far below a bomb.
const MAX_ALLOC: u64 = 256 * 1024 * 1024;
/// Most pixels of one decoded image: 64 MP is 256 MiB as RGBA.
pub(crate) const MAX_PIXELS: u64 = 64 * 1000 * 1000;
/// Most frames of an animated GIF.
const MAX_GIF_FRAMES: usize = 500;
/// Most pixels over all frames of an animated GIF, before downscaling.
const MAX_GIF_PIXELS: u64 = 256 * 1024 * 1024;
/// Encoder effort for GIF palettes (1 is best and slowest, 30 fastest).
const GIF_SPEED: i32 = 10;

/// The format of a stored image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stored {
    /// Lossy, for photos.
    Jpeg,
    /// Lossless, for PNG and WebP sources and for stills of GIFs.
    Webp,
    /// Animated GIFs stay GIFs.
    Gif,
}

impl Stored {
    /// File name extension.
    pub(crate) fn ext(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Gif => "gif",
        }
    }

    /// Stored `mime` column.
    pub(crate) fn mime(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Gif => "image/gif",
        }
    }
}

/// A re-encoded image.
pub(crate) struct Encoded {
    /// The file bytes.
    pub(crate) bytes: Vec<u8>,
    /// Their format.
    pub(crate) stored: Stored,
    /// Pixel width.
    pub(crate) width: u32,
    /// Pixel height.
    pub(crate) height: u32,
}

/// What an image is for.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Target {
    /// A post attachment, a video poster or a link card: long edge at most [`MAX_EDGE`].
    Post,
    /// Rotated, cropped and scaled to 400×400.
    Avatar(Option<CropRect>),
    /// Rotated, cropped and scaled to 1500×500.
    Banner(Option<CropRect>),
    /// Centered and scaled to 400×400, for clients that send no crop (the Mastodon API).
    AvatarFill,
    /// Centered and scaled to 1500×500, for clients that send no crop (the Mastodon API).
    BannerFill,
    /// A custom emote: long edge at most [`EMOTE_EDGE`], animated GIFs stay animated.
    Emote,
}

/// Why an image could not be processed.
#[derive(Debug)]
pub(crate) enum Failure {
    /// Not a PNG, JPEG, GIF or WebP, or damaged, or over the decoder limits.
    NotAnImage,
    /// The crop is missing, outside the image or the rotation is not a right angle.
    BadCrop,
    /// Encoding failed.
    Encode(ImageError),
}

fn decode(_: ImageError) -> Failure {
    Failure::NotAnImage
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}

/// Decodes `bytes`, trusting the content and not the declared type, and encodes it for `target`.
pub(crate) fn process(bytes: &[u8], target: Target) -> Result<Encoded, Failure> {
    let format = image::guess_format(bytes).map_err(decode)?;
    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP
    ) {
        return Err(Failure::NotAnImage);
    }
    if format == ImageFormat::Gif && matches!(target, Target::Post | Target::Emote) {
        let max_edge = if matches!(target, Target::Emote) {
            EMOTE_EDGE
        } else {
            MAX_EDGE
        };
        return animated_gif(bytes, max_edge);
    }
    let image = decode_still(bytes, format)?;
    let stored = if format == ImageFormat::Jpeg {
        Stored::Jpeg
    } else {
        Stored::Webp
    };
    let image = match target {
        Target::Post => fit(image, MAX_EDGE),
        Target::Emote => fit(image, EMOTE_EDGE),
        Target::Avatar(crop) => cut(image, crop, AVATAR)?,
        Target::Banner(crop) => cut(image, crop, BANNER)?,
        Target::AvatarFill => image.resize_to_fill(AVATAR.0, AVATAR.1, FilterType::Lanczos3),
        Target::BannerFill => image.resize_to_fill(BANNER.0, BANNER.1, FilterType::Lanczos3),
    };
    encode(&image, stored)
}

/// Decodes the first frame and applies the EXIF orientation, which the re-encode would drop.
fn decode_still(bytes: &[u8], format: ImageFormat) -> Result<DynamicImage, Failure> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits());
    let mut decoder = reader.into_decoder().map_err(decode)?;
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(Failure::NotAnImage);
    }
    // `from_decoder` never reserves the output buffer, so the allocation limit would go unchecked.
    limits().reserve(decoder.total_bytes()).map_err(decode)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder).map_err(decode)?;
    image.apply_orientation(orientation);
    Ok(image)
}

/// Scales `image` down so its long edge is at most `max_edge`. Never scales up.
fn fit(image: DynamicImage, max_edge: u32) -> DynamicImage {
    if image.width().max(image.height()) > max_edge {
        image.resize(max_edge, max_edge, FilterType::Lanczos3)
    } else {
        image
    }
}

/// Rotates clockwise, crops and scales to exactly `size`.
fn cut(
    image: DynamicImage,
    crop: Option<CropRect>,
    (width, height): (u32, u32),
) -> Result<DynamicImage, Failure> {
    let crop = crop.ok_or(Failure::BadCrop)?;
    let image = match crop.rotate {
        0 => image,
        90 => image.rotate90(),
        180 => image.rotate180(),
        270 => image.rotate270(),
        _ => return Err(Failure::BadCrop),
    };
    let inside =
        |start: u32, len: u32, limit: u32| start.checked_add(len).is_some_and(|end| end <= limit);
    if crop.w == 0
        || crop.h == 0
        || !inside(crop.x, crop.w, image.width())
        || !inside(crop.y, crop.h, image.height())
    {
        return Err(Failure::BadCrop);
    }
    Ok(image
        .crop_imm(crop.x, crop.y, crop.w, crop.h)
        .resize_to_fill(width, height, FilterType::Lanczos3))
}

fn encode(image: &DynamicImage, stored: Stored) -> Result<Encoded, Failure> {
    let (width, height) = (image.width(), image.height());
    let mut bytes = Vec::new();
    if stored == Stored::Jpeg {
        let rgb = image.to_rgb8();
        JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY).encode(
            rgb.as_raw(),
            width,
            height,
            ExtendedColorType::Rgb8,
        )
    } else {
        let encoder = WebPEncoder::new_lossless(&mut bytes);
        if image.color().has_alpha() {
            let rgba = image.to_rgba8();
            encoder.encode(rgba.as_raw(), width, height, ExtendedColorType::Rgba8)
        } else {
            let rgb = image.to_rgb8();
            encoder.encode(rgb.as_raw(), width, height, ExtendedColorType::Rgb8)
        }
    }
    .map_err(Failure::Encode)?;
    Ok(Encoded {
        bytes,
        stored,
        width,
        height,
    })
}

/// Re-encodes every frame, scaled to `max_edge` when larger, as an endlessly looping GIF.
fn animated_gif(bytes: &[u8], max_edge: u32) -> Result<Encoded, Failure> {
    let mut decoder = GifDecoder::new(Cursor::new(bytes)).map_err(decode)?;
    decoder.set_limits(limits()).map_err(decode)?;
    let (source_w, source_h) = decoder.dimensions();
    if u64::from(source_w) * u64::from(source_h) > MAX_PIXELS {
        return Err(Failure::NotAnImage);
    }
    let (width, height) = fit_dimensions(source_w, source_h, max_edge);
    let scaled = (width, height) != (source_w, source_h);
    let mut out = Vec::new();
    let mut frames = 0_usize;
    {
        let mut encoder = GifEncoder::new_with_speed(&mut out, GIF_SPEED);
        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(Failure::Encode)?;
        for frame in decoder.into_frames() {
            let frame = frame.map_err(decode)?;
            frames += 1;
            if frames > MAX_GIF_FRAMES
                || u64::from(source_w) * u64::from(source_h) * frames as u64 > MAX_GIF_PIXELS
            {
                return Err(Failure::NotAnImage);
            }
            let delay = frame.delay();
            let buffer = frame.into_buffer();
            let buffer = if scaled {
                imageops::resize(&buffer, width, height, FilterType::CatmullRom)
            } else {
                buffer
            };
            encoder
                .encode_frame(Frame::from_parts(buffer, 0, 0, delay))
                .map_err(Failure::Encode)?;
        }
    }
    if frames == 0 {
        return Err(Failure::NotAnImage);
    }
    Ok(Encoded {
        bytes: out,
        stored: Stored::Gif,
        width,
        height,
    })
}

/// The size of `width` × `height` scaled to fit `max_edge`, keeping the aspect ratio.
fn fit_dimensions(width: u32, height: u32, max_edge: u32) -> (u32, u32) {
    let long = width.max(height);
    if long <= max_edge {
        return (width, height);
    }
    let scale = |edge: u32| {
        let scaled =
            (u64::from(edge) * u64::from(max_edge) + u64::from(long) / 2) / u64::from(long);
        u32::try_from(scaled).unwrap_or(max_edge).max(1)
    };
    (scale(width), scale(height))
}
