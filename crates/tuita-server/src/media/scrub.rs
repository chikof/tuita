//! Metadata scrub for stored videos.
//!
//! An uploaded or federated video keeps every byte of its container, and with them GPS fixes,
//! device make and model, author and tool tags, and XMP packets. The scrub rewrites the file in
//! place without changing its size: a metadata box or element is retyped to a filler (`free` in
//! MP4 and MOV, `Void` in WebM) and its payload is overwritten with zero bytes. Offsets of every
//! box, element and sample stay valid, so playback is unchanged and nothing is remuxed.
//!
//! The walkers read headers only and seek over payloads, so a 1 GiB file is never loaded. They
//! stop at the first structure they cannot follow (a truncated box, a size smaller than its
//! header, a size past the end, too deep a nesting, too many boxes) and leave the rest of the
//! file as it is: a file the scrub cannot parse is still stored.

use std::fs::OpenOptions;
use std::io::{self, Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

use super::video::Container;

/// Deepest box nesting the walker follows. Real files nest `moov` > `trak` > `mdia` > `minf`.
const MAX_DEPTH: u8 = 16;
/// Most boxes or elements one scrub visits.
const MAX_NODES: u32 = 100_000;
/// Bytes written per call when a payload is zeroed.
const CHUNK: usize = 8192;
const ZEROS: [u8; CHUNK] = [0; CHUNK];

/// Box types whose payload is metadata: XMP and camera blobs (`uuid`, `XMP_`), the `udta` user
/// data with `©xyz`, `©mak`, `©mod`, `©swr`, `loci` and `Xtra`, and the `meta` item list.
const METADATA_BOXES: [&[u8; 4]; 4] = [b"udta", b"meta", b"uuid", b"XMP_"];
/// Box types holding the boxes the walker looks into.
const CONTAINER_BOXES: [&[u8; 4]; 3] = [b"moov", b"trak", b"mdia"];
/// Box types that start with a version, flags, then creation and modification times.
const TIMED_BOXES: [&[u8; 4]; 3] = [b"mvhd", b"tkhd", b"mdhd"];

/// The EBML ID of the `Segment` element that holds everything but the EBML header.
const SEGMENT: u32 = 0x1853_8067;
/// The ID of `Info`, which holds the title, the date and the names of the tools.
const INFO: u32 = 0x1549_a966;
/// The ID of `Tags`, the free-form tags of a file.
const TAGS: u32 = 0x1254_c367;
/// The IDs of the `Info` children that name the file or the tools that wrote it: `Title`,
/// `DateUTC`, `MuxingApp` and `WritingApp`.
const INFO_METADATA: [u32; 4] = [0x7ba9, 0x4461, 0x4d80, 0x5741];
/// The ID of `Void`, the filler element.
const VOID: u8 = 0xec;

/// Why a walk stopped before the end of the file.
#[derive(Debug)]
enum Stop {
    /// The structure is not one the walker can follow; the rest of the file stays as it is.
    Malformed(&'static str),
    /// The file could not be read or written.
    Io(io::Error),
}

impl From<io::Error> for Stop {
    fn from(err: io::Error) -> Self {
        if err.kind() == ErrorKind::UnexpectedEof {
            Self::Malformed("truncated file")
        } else {
            Self::Io(err)
        }
    }
}

type Walk = Result<(), Stop>;

/// Scrubs the metadata of the `container` video behind `file`, in place and without changing its
/// length.
///
/// Content the walker cannot follow ends the scrub quietly: what was scrubbed before stays, the
/// rest is untouched, and the result is `Ok`.
///
/// # Errors
///
/// Reading or writing `file` failed. The file may then be partly scrubbed.
pub(crate) fn scrub<F: Read + Write + Seek>(file: &mut F, container: Container) -> io::Result<()> {
    let len = file.seek(SeekFrom::End(0))?;
    let mut scrubber = Scrubber { file, nodes: 0 };
    let walked = match container {
        Container::Mp4 | Container::Mov => scrubber.walk_boxes(0, len, 0),
        Container::Webm => scrubber.walk_ebml(len),
    };
    match walked {
        Ok(()) => Ok(()),
        Err(Stop::Malformed(reason)) => {
            tracing::debug!(reason, nodes = scrubber.nodes, "video scrub stopped early");
            Ok(())
        }
        Err(Stop::Io(err)) => Err(err),
    }
}

/// Scrubs the video file at `path` in place.
///
/// # Errors
///
/// The file could not be opened, read or written.
pub(crate) fn scrub_file(path: &Path, container: Container) -> io::Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    scrub(&mut file, container)
}

/// Scrubs `bytes`, a whole video, and returns them with the same length.
pub(crate) fn scrub_bytes(bytes: Vec<u8>, container: Container) -> Vec<u8> {
    let mut cursor = Cursor::new(bytes);
    // Memory has no I/O failure to report; log it rather than hide it if that ever changes.
    if let Err(err) = scrub(&mut cursor, container) {
        tracing::debug!(%err, "video scrub failed in memory");
    }
    cursor.into_inner()
}

struct Scrubber<'a, F> {
    file: &'a mut F,
    /// Boxes or elements visited so far.
    nodes: u32,
}

impl<F: Read + Write + Seek> Scrubber<'_, F> {
    /// Counts one more box or element, refusing past [`MAX_NODES`].
    fn count(&mut self) -> Walk {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > MAX_NODES {
            return Err(Stop::Malformed("too many boxes"));
        }
        Ok(())
    }

    fn read_at(&mut self, pos: u64, buf: &mut [u8]) -> Walk {
        self.file.seek(SeekFrom::Start(pos))?;
        self.file.read_exact(buf)?;
        Ok(())
    }

    fn write_at(&mut self, pos: u64, bytes: &[u8]) -> Walk {
        self.file.seek(SeekFrom::Start(pos))?;
        self.file.write_all(bytes)?;
        Ok(())
    }

    /// Overwrites `len` bytes from `pos` with zeros.
    fn zero(&mut self, pos: u64, mut len: u64) -> Walk {
        self.file.seek(SeekFrom::Start(pos))?;
        while len > 0 {
            let step = usize::try_from(len).unwrap_or(CHUNK).min(CHUNK);
            self.file.write_all(&ZEROS[..step])?;
            len -= step as u64;
        }
        Ok(())
    }

    /// Walks the boxes in `start..end`, which sit `depth` containers deep.
    fn walk_boxes(&mut self, start: u64, end: u64, depth: u8) -> Walk {
        if depth > MAX_DEPTH {
            return Err(Stop::Malformed("boxes nested too deep"));
        }
        let mut pos = start;
        while pos < end {
            self.count()?;
            let head = self.box_head(pos, end)?;
            let body = pos + head.header;
            let next = pos + head.size;
            if METADATA_BOXES.contains(&&head.kind) {
                self.write_at(pos + 4, b"free")?;
                self.zero(body, head.size - head.header)?;
            } else if CONTAINER_BOXES.contains(&&head.kind) {
                self.walk_boxes(body, next, depth + 1)?;
            } else if TIMED_BOXES.contains(&&head.kind) {
                self.zero_times(body, next)?;
            }
            pos = next;
        }
        Ok(())
    }

    /// Reads the header of the box at `pos`, which lies in a parent ending at `end`, and checks
    /// the box fits.
    fn box_head(&mut self, pos: u64, end: u64) -> Result<BoxHead, Stop> {
        let room = end - pos;
        let mut buf = [0_u8; 8];
        if room < buf.len() as u64 {
            return Err(Stop::Malformed("box header cut short"));
        }
        self.read_at(pos, &mut buf)?;
        let [s0, s1, s2, s3, k0, k1, k2, k3] = buf;
        let kind = [k0, k1, k2, k3];
        let (header, size) = match u32::from_be_bytes([s0, s1, s2, s3]) {
            0 => (8, room),
            1 => {
                let mut wide = [0_u8; 8];
                if room < 16 {
                    return Err(Stop::Malformed("box header cut short"));
                }
                self.read_at(pos + 8, &mut wide)?;
                (16, u64::from_be_bytes(wide))
            }
            size => (8, u64::from(size)),
        };
        if size < header {
            return Err(Stop::Malformed("box smaller than its header"));
        }
        if size > room {
            return Err(Stop::Malformed("box past the end of its parent"));
        }
        Ok(BoxHead { kind, header, size })
    }

    /// Zeroes the creation and modification times of the `mvhd`, `tkhd` or `mdhd` whose payload
    /// is `body..end`. Version 1 stores them as 64-bit values, version 0 as 32-bit.
    fn zero_times(&mut self, body: u64, end: u64) -> Walk {
        let mut version = [0_u8; 1];
        if end - body < 4 {
            return Err(Stop::Malformed("box too small for its fields"));
        }
        self.read_at(body, &mut version)?;
        let times: u64 = match version[0] {
            0 => 8,
            1 => 16,
            _ => return Ok(()),
        };
        if end - body < 4 + times {
            return Err(Stop::Malformed("box too small for its fields"));
        }
        self.zero(body + 4, times)
    }
}

impl<F: Read + Write + Seek> Scrubber<'_, F> {
    /// Walks the top-level elements of a Matroska file `len` bytes long.
    fn walk_ebml(&mut self, len: u64) -> Walk {
        let mut pos = 0;
        while pos < len {
            self.count()?;
            let el = self.element(pos, len)?;
            if el.id == SEGMENT {
                let body = pos + el.header;
                // A segment of unknown size, or one cut short, runs to the end of the file.
                let end = el
                    .size
                    .map_or(len, |size| body.saturating_add(size).min(len));
                self.walk_segment(body, end)?;
                pos = end;
            } else {
                pos += el.extent(len - pos)?;
            }
        }
        Ok(())
    }

    /// Walks the children of a `Segment` whose body is `start..end`.
    fn walk_segment(&mut self, start: u64, end: u64) -> Walk {
        let mut pos = start;
        while pos < end {
            self.count()?;
            let el = self.element(pos, end)?;
            let total = el.extent(end - pos)?;
            match el.id {
                TAGS => self.void(pos, total)?,
                INFO => self.walk_info(pos + el.header, pos + total)?,
                _ => {}
            }
            pos += total;
        }
        Ok(())
    }

    /// Walks the children of an `Info` element whose body is `start..end`.
    fn walk_info(&mut self, start: u64, end: u64) -> Walk {
        let mut pos = start;
        while pos < end {
            self.count()?;
            let el = self.element(pos, end)?;
            let total = el.extent(end - pos)?;
            if INFO_METADATA.contains(&el.id) {
                self.void(pos, total)?;
            }
            pos += total;
        }
        Ok(())
    }

    /// Reads the element header at `pos`, which lies in a parent ending at `end`.
    fn element(&mut self, pos: u64, end: u64) -> Result<Element, Stop> {
        let mut buf = [0_u8; Element::MAX_HEADER];
        let want = usize::try_from(end - pos).map_or(buf.len(), |room| room.min(buf.len()));
        self.read_at(pos, &mut buf[..want])?;
        Element::parse(&buf[..want])
    }

    /// Turns the `total` bytes of the element at `pos` into a `Void` element of the same length
    /// with a zeroed payload.
    ///
    /// A size field may be written longer than it needs to be, so a `Void` fits any element of
    /// two bytes or more, and every element has an ID and a size field.
    fn void(&mut self, pos: u64, total: u64) -> Walk {
        for size_len in 1..=8_u32 {
            let Some(payload) = total.checked_sub(1 + u64::from(size_len)) else {
                return Ok(());
            };
            // The all-ones value of a size field means "unknown".
            if payload < (1 << (7 * size_len)) - 1 {
                let size = (payload | (1 << (7 * size_len))).to_be_bytes();
                let mut head = [VOID; 9];
                let size_len = size_len as usize;
                head[1..=size_len].copy_from_slice(&size[size.len() - size_len..]);
                self.write_at(pos, &head[..=size_len])?;
                return self.zero(pos + 1 + size_len as u64, payload);
            }
        }
        Ok(())
    }
}

/// An EBML element header: its ID, the header length, and the payload length unless the size
/// is "unknown".
#[derive(Debug, PartialEq, Eq)]
struct Element {
    id: u32,
    header: u64,
    size: Option<u64>,
}

impl Element {
    /// An ID takes at most 4 bytes and a size at most 8.
    const MAX_HEADER: usize = 12;

    /// Parses the header at the start of `bytes`.
    fn parse(bytes: &[u8]) -> Result<Self, Stop> {
        let cut = || Stop::Malformed("element header cut short");
        let first = *bytes.first().ok_or_else(cut)?;
        let id_len = vint_len(first)
            .filter(|len| *len <= 4)
            .ok_or(Stop::Malformed("bad element id"))?;
        let id = bytes
            .get(..id_len)
            .ok_or_else(cut)?
            .iter()
            .fold(0_u32, |id, byte| (id << 8) | u32::from(*byte));
        let size_first = *bytes.get(id_len).ok_or_else(cut)?;
        let size_len = vint_len(size_first).ok_or(Stop::Malformed("bad element size"))?;
        let size_bytes = bytes.get(id_len..id_len + size_len).ok_or_else(cut)?;
        let value = size_bytes.iter().skip(1).fold(
            u64::from(size_first) & ((1 << (8 - size_len)) - 1),
            |size, byte| (size << 8) | u64::from(*byte),
        );
        let unknown = value == (1 << (7 * size_len)) - 1;
        Ok(Self {
            id,
            header: (id_len + size_len) as u64,
            size: (!unknown).then_some(value),
        })
    }

    /// The whole length of the element when its parent has `room` bytes left from its start.
    fn extent(&self, room: u64) -> Result<u64, Stop> {
        let size = self
            .size
            .ok_or(Stop::Malformed("element of unknown size"))?;
        match self.header.checked_add(size) {
            Some(total) if total <= room => Ok(total),
            _ => Err(Stop::Malformed("element past the end of its parent")),
        }
    }
}

/// The length in bytes of the variable-length integer that starts with `first`.
fn vint_len(first: u8) -> Option<usize> {
    (first != 0).then(|| first.leading_zeros() as usize + 1)
}

/// A box header: its four-character type, the header length (8, or 16 with a `largesize`), and
/// the whole box length.
struct BoxHead {
    kind: [u8; 4],
    header: u64,
    size: u64,
}

#[cfg(test)]
mod test {
    use super::*;

    /// A handle that records the furthest position it was moved to.
    struct Watched {
        inner: Cursor<Vec<u8>>,
        reach: u64,
    }

    impl Watched {
        fn new(bytes: Vec<u8>) -> Self {
            Self {
                inner: Cursor::new(bytes),
                reach: 0,
            }
        }

        fn note(&mut self) {
            self.reach = self.reach.max(self.inner.position());
        }
    }

    impl Read for Watched {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let n = self.inner.read(buf)?;
            self.note();
            Ok(n)
        }
    }

    impl Write for Watched {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let n = self.inner.write(buf)?;
            self.note();
            Ok(n)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    impl Seek for Watched {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            let at = self.inner.seek(pos)?;
            self.note();
            Ok(at)
        }
    }

    /// Scrubs `input`, checking it returns `Ok`, keeps its length and never reaches past its end.
    fn run(input: &[u8], container: Container) -> Vec<u8> {
        let mut file = Watched::new(input.to_vec());
        scrub(&mut file, container).expect("a scrub of memory returns");
        let out = file.inner.into_inner();
        assert_eq!(out.len(), input.len());
        assert!(file.reach <= input.len() as u64, "reached {}", file.reach);
        out
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }

    /// Deterministic noise.
    fn noise(seed: u64, len: usize) -> Vec<u8> {
        let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 24) as u8
            })
            .collect()
    }

    // ---- MP4 and MOV ----

    const GPS: &[u8] = b"+52.5200+013.4050/";
    const ISO6709_KEY: &[u8] = b"com.apple.quicktime.location.ISO6709";
    const MAKE_KEY: &[u8] = b"com.apple.quicktime.make";
    const MODEL_KEY: &[u8] = b"com.apple.quicktime.model";
    const MODEL: &[u8] = b"iPhone 15 Pro";
    const CAMERA_MAKE: &[u8] = b"Samsung";
    const CAMERA_MODEL: &[u8] = b"SM-S918B";
    const TRAK_DEVICE: &[u8] = b"Pixel 8 Camera";
    const XMP: &[u8] = b"<x:xmpmeta><rdf:Description exif:GPSLatitude=\"52.52\"/></x:xmpmeta>";
    const SECRETS: [&[u8]; 9] = [
        GPS,
        ISO6709_KEY,
        MAKE_KEY,
        MODEL_KEY,
        MODEL,
        CAMERA_MAKE,
        CAMERA_MODEL,
        TRAK_DEVICE,
        b"GPSLatitude",
    ];
    const MDAT: &[u8] = b"MDAT-PAYLOAD-0123456789";
    const STCO_OFFSET: u32 = 0x1234;

    fn bx(kind: &[u8], payload: &[u8]) -> Vec<u8> {
        let size = u32::try_from(8 + payload.len()).expect("small box");
        [&size.to_be_bytes()[..], kind, payload].concat()
    }

    /// A box in the 64-bit `largesize` form.
    fn bx64(kind: &[u8], payload: &[u8]) -> Vec<u8> {
        let size = 16 + payload.len() as u64;
        [&1_u32.to_be_bytes()[..], kind, &size.to_be_bytes(), payload].concat()
    }

    /// A box whose size field is 0: it runs to the end of the file.
    fn bx0(kind: &[u8], payload: &[u8]) -> Vec<u8> {
        [&0_u32.to_be_bytes()[..], kind, payload].concat()
    }

    fn ftyp() -> Vec<u8> {
        bx(b"ftyp", b"mp42\0\0\0\0mp42isom")
    }

    /// An `mvhd`, `tkhd` or `mdhd`: version, flags, creation and modification time, a timescale and
    /// a duration, then filler.
    fn timed(kind: &[u8], version: u8) -> Vec<u8> {
        let mut payload = vec![version, 0, 0, 0];
        if version == 1 {
            payload.extend(0x0123_4567_89ab_cdef_u64.to_be_bytes());
            payload.extend(0x0fed_cba9_8765_4321_u64.to_be_bytes());
            payload.extend(1000_u32.to_be_bytes());
            payload.extend(5000_u64.to_be_bytes());
        } else {
            payload.extend(0x0123_4567_u32.to_be_bytes());
            payload.extend(0x0765_4321_u32.to_be_bytes());
            payload.extend(1000_u32.to_be_bytes());
            payload.extend(5000_u32.to_be_bytes());
        }
        payload.extend([0x11; 40]);
        bx(kind, &payload)
    }

    /// A QuickTime user-data atom: length, language, text.
    fn atom(kind: [u8; 4], text: &[u8]) -> Vec<u8> {
        let len = u16::try_from(text.len()).expect("short text");
        bx(
            &kind,
            &[&len.to_be_bytes()[..], &[0x55, 0xc4], text].concat(),
        )
    }

    fn stco() -> Vec<u8> {
        let payload = [0_u32, 1, STCO_OFFSET].map(u32::to_be_bytes).concat();
        bx(b"stco", &payload)
    }

    fn trak(version: u8) -> Vec<u8> {
        let minf = bx(b"minf", &bx(b"stbl", &stco()));
        let mdia = bx(
            b"mdia",
            &[timed(b"mdhd", version), bx(b"hdlr", &[0; 24]), minf].concat(),
        );
        let udta = bx(b"udta", &atom(*b"\xa9xyz", TRAK_DEVICE));
        bx(b"trak", &[timed(b"tkhd", version), mdia, udta].concat())
    }

    /// The QuickTime `meta` box: handler, keys, item list.
    fn quicktime_meta() -> Vec<u8> {
        let keys = [ISO6709_KEY, MAKE_KEY, MODEL_KEY]
            .iter()
            .flat_map(|key| bx(b"mdta", key))
            .collect::<Vec<_>>();
        let items = [GPS, &b"Apple"[..], MODEL]
            .iter()
            .flat_map(|value| bx(b"data", &[&[0, 0, 0, 1, 0, 0, 0, 0][..], *value].concat()))
            .collect::<Vec<_>>();
        bx(
            b"meta",
            &[
                bx(b"hdlr", b"\0\0\0\0\0\0\0\0mdta").as_slice(),
                bx(b"keys", &[&[0, 0, 0, 0, 0, 0, 0, 3][..], &keys].concat()).as_slice(),
                bx(b"ilst", &items).as_slice(),
            ]
            .concat(),
        )
    }

    /// The children of `moov`.
    fn moov_payload(version: u8) -> Vec<u8> {
        let udta = bx(
            b"udta",
            &[
                atom(*b"\xa9xyz", GPS),
                atom(*b"\xa9mak", CAMERA_MAKE),
                atom(*b"\xa9mod", CAMERA_MODEL),
            ]
            .concat(),
        );
        [
            timed(b"mvhd", version),
            trak(version),
            udta,
            quicktime_meta(),
            bx(b"uuid", &[&[7_u8; 16][..], b"camera blob ", MODEL].concat()),
        ]
        .concat()
    }

    #[derive(Clone, Copy)]
    enum Layout {
        /// 32-bit sizes everywhere.
        Plain,
        /// `moov` and `mdat` in the `largesize` form, the last box with size 0.
        Wide,
    }

    /// An MP4 with `moov` before or after `mdat`, and an XMP `uuid` box last.
    fn sample(layout: Layout, version: u8, moov_first: bool) -> Vec<u8> {
        let xmp_uuid = [&[0xbe_u8; 16][..], XMP].concat();
        let (moov, mdat, xmp) = match layout {
            Layout::Plain => (
                bx(b"moov", &moov_payload(version)),
                bx(b"mdat", MDAT),
                bx(b"uuid", &xmp_uuid),
            ),
            Layout::Wide => (
                bx64(b"moov", &moov_payload(version)),
                bx64(b"mdat", MDAT),
                bx0(b"uuid", &xmp_uuid),
            ),
        };
        if moov_first {
            [ftyp(), moov, mdat, xmp].concat()
        } else {
            [ftyp(), mdat, moov, xmp].concat()
        }
    }

    #[derive(Debug)]
    struct Item {
        depth: usize,
        kind: [u8; 4],
        offset: usize,
        header: usize,
        size: usize,
    }

    const WALKED: [&[u8; 4]; 5] = [b"moov", b"trak", b"mdia", b"minf", b"stbl"];

    /// Every box of a valid file, down through `moov`, `trak`, `mdia`, `minf` and `stbl`.
    fn flat(bytes: &[u8]) -> Vec<Item> {
        fn walk(bytes: &[u8], start: usize, end: usize, depth: usize, out: &mut Vec<Item>) {
            let mut pos = start;
            while pos < end {
                let size32 = u32::from_be_bytes(bytes[pos..pos + 4].try_into().expect("4 bytes"));
                let kind: [u8; 4] = bytes[pos + 4..pos + 8].try_into().expect("4 bytes");
                let (header, size) = match size32 {
                    0 => (8, end - pos),
                    1 => {
                        let wide = bytes[pos + 8..pos + 16].try_into().expect("8 bytes");
                        (16, usize::try_from(u64::from_be_bytes(wide)).expect("fits"))
                    }
                    n => (8, n as usize),
                };
                assert!(size >= header && pos + size <= end, "a box at {pos}");
                out.push(Item {
                    depth,
                    kind,
                    offset: pos,
                    header,
                    size,
                });
                if WALKED.contains(&&kind) {
                    walk(bytes, pos + header, pos + size, depth + 1, out);
                }
                pos += size;
            }
        }
        let mut out = Vec::new();
        walk(bytes, 0, bytes.len(), 0, &mut out);
        out
    }

    fn check_mp4(input: &[u8], version: u8) {
        for secret in SECRETS {
            assert!(contains(input, secret), "the fixture holds {secret:?}");
        }
        let out = run(input, Container::Mp4);
        for secret in SECRETS {
            assert!(
                !contains(&out, secret),
                "{:?} survived",
                String::from_utf8_lossy(secret)
            );
        }

        let before = flat(input);
        let after = flat(&out);
        assert_eq!(before.len(), after.len());
        for (old, new) in before.iter().zip(&after) {
            assert_eq!(
                (old.depth, old.offset, old.header, old.size),
                (new.depth, new.offset, new.header, new.size)
            );
            let body = new.offset + new.header..new.offset + new.size;
            if [b"udta", b"meta", b"uuid"].contains(&&old.kind) {
                assert_eq!(&new.kind, b"free");
                assert!(out[body].iter().all(|byte| *byte == 0), "payload zeroed");
                continue;
            }
            assert_eq!(old.kind, new.kind);
            if [b"mvhd", b"tkhd", b"mdhd"].contains(&&old.kind) {
                // Only the times (after the version and flags) change; timescale and duration
                // stay.
                let times = body.start + 4..body.start + 4 + if version == 1 { 16 } else { 8 };
                assert!(out[times.clone()].iter().all(|byte| *byte == 0));
                assert!(input[times.clone()].iter().any(|byte| *byte != 0));
                assert_eq!(out[body.start..times.start], input[body.start..times.start]);
                assert_eq!(out[times.end..body.end], input[times.end..body.end]);
            } else if !WALKED.contains(&&old.kind) {
                let whole = new.offset..new.offset + new.size;
                assert_eq!(out[whole.clone()], input[whole], "{:?}", old.kind);
            }
        }
        let stco = after
            .iter()
            .find(|item| &item.kind == b"stco")
            .expect("stco");
        let offset = &out[stco.offset + stco.size - 4..stco.offset + stco.size];
        assert_eq!(offset, STCO_OFFSET.to_be_bytes());
        assert!(contains(&out, MDAT));
    }

    #[test]
    fn an_mp4_loses_its_location_device_and_xmp_and_keeps_its_structure() {
        check_mp4(&sample(Layout::Plain, 0, true), 0);
    }

    #[test]
    fn version_1_times_are_zeroed_too() {
        check_mp4(&sample(Layout::Plain, 1, true), 1);
    }

    #[test]
    fn a_mov_with_largesize_boxes_and_a_size_zero_last_box_is_scrubbed() {
        let input = sample(Layout::Wide, 0, true);
        let tail = flat(&input).pop().expect("a last box");
        assert_eq!(&input[tail.offset..tail.offset + 4], [0, 0, 0, 0], "size 0");
        check_mp4(&input, 0);
        assert_eq!(
            run(&input, Container::Mov),
            run(&input, Container::Mp4),
            "MOV and MP4 walk alike"
        );
    }

    #[test]
    fn a_moov_after_mdat_is_scrubbed() {
        check_mp4(&sample(Layout::Plain, 0, false), 0);
        check_mp4(&sample(Layout::Wide, 1, false), 1);
    }

    #[test]
    fn a_top_level_meta_and_xmp_box_are_scrubbed() {
        let input = [
            ftyp(),
            bx(b"meta", GPS),
            bx(b"XMP_", b"GPSLatitude"),
            bx(b"udta", CAMERA_MAKE),
            bx(b"free", b"keep me"),
            bx(b"mdat", MDAT),
        ]
        .concat();
        let out = run(&input, Container::Mp4);
        let kinds: Vec<_> = flat(&out).iter().map(|item| item.kind).collect();
        assert_eq!(
            kinds,
            [b"ftyp", b"free", b"free", b"free", b"free", b"mdat"].map(|kind| *kind)
        );
        for secret in [GPS, &b"GPSLatitude"[..], CAMERA_MAKE] {
            assert!(!contains(&out, secret));
        }
        assert!(contains(&out, b"keep me"), "a `free` box is not touched");
    }

    #[test]
    fn scrubbing_twice_changes_nothing_more() {
        let once = run(&sample(Layout::Plain, 0, true), Container::Mp4);
        assert_eq!(run(&once, Container::Mp4), once);
    }

    #[test]
    fn a_large_payload_is_zeroed_past_one_chunk() {
        let big = vec![0xcd; CHUNK * 3 + 17];
        let input = [ftyp(), bx(b"uuid", &big), bx(b"mdat", MDAT)].concat();
        let out = run(&input, Container::Mp4);
        assert!(!out.contains(&0xcd));
        assert!(contains(&out, MDAT));
    }

    /// `head` scrubbed, then `bad` and `rest`: everything from `bad` on must be as it was.
    fn assert_stops_at_bad(bad: &[u8], rest: &[u8]) {
        let head = [ftyp(), bx(b"udta", GPS)].concat();
        let input = [head.clone(), bad.to_vec(), rest.to_vec()].concat();
        let out = run(&input, Container::Mp4);
        assert!(
            !contains(&out[..head.len()], GPS),
            "the part before the damage is scrubbed"
        );
        assert_eq!(
            out[head.len()..],
            input[head.len()..],
            "the rest is untouched"
        );
    }

    #[test]
    fn a_truncated_box_stops_the_scrub() {
        let moov = bx(b"moov", &[0x77; 1000]);
        assert_stops_at_bad(&moov[..50], &[]);
    }

    #[test]
    fn a_box_smaller_than_its_header_stops_the_scrub() {
        for size in [2_u32, 4, 7] {
            let bad = [&size.to_be_bytes()[..], b"moov"].concat();
            assert_stops_at_bad(&bad, &bx(b"udta", CAMERA_MAKE));
        }
    }

    #[test]
    fn a_box_past_the_end_of_the_file_stops_the_scrub() {
        let bad = [&0x7fff_ffff_u32.to_be_bytes()[..], b"udta", GPS].concat();
        assert_stops_at_bad(&bad, &[]);
        let bad = [&0xffff_ffff_u32.to_be_bytes()[..], b"moov", &[1; 64]].concat();
        assert_stops_at_bad(&bad, &[]);
    }

    #[test]
    fn a_wide_box_with_a_bad_size_stops_the_scrub() {
        let past = [&1_u32.to_be_bytes()[..], b"moov", &u64::MAX.to_be_bytes()].concat();
        assert_stops_at_bad(&past, &[]);
        let small = [&1_u32.to_be_bytes()[..], b"moov", &9_u64.to_be_bytes()].concat();
        assert_stops_at_bad(&small, &[]);
        let cut = [&1_u32.to_be_bytes()[..], b"moov", &[0; 4]].concat();
        assert_stops_at_bad(&cut, &[]);
    }

    #[test]
    fn trailing_bytes_shorter_than_a_header_stop_the_scrub() {
        assert_stops_at_bad(&[0, 0, 0], &[]);
    }

    #[test]
    fn a_box_whose_children_overrun_it_stops_the_scrub() {
        // The child `udta` claims more than its `moov` holds, so the next `udta` is not reached.
        let child = [&100_u32.to_be_bytes()[..], b"udta", GPS].concat();
        let moov = bx(b"moov", &child);
        assert_stops_at_bad(&moov, &bx(b"udta", CAMERA_MAKE));
    }

    #[test]
    fn a_header_box_too_short_for_its_times_stops_the_scrub() {
        let moov = bx(b"moov", &bx(b"mvhd", &[0, 0, 0, 0, 1, 2]));
        assert_stops_at_bad(&moov, &bx(b"udta", CAMERA_MAKE));
        let moov = bx(b"moov", &bx(b"mvhd", &[1, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]));
        assert_stops_at_bad(&moov, &bx(b"udta", CAMERA_MAKE));
    }

    #[test]
    fn a_header_box_with_an_unknown_version_keeps_its_bytes() {
        let mvhd = bx(
            b"mvhd",
            &[9, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        );
        let input = [ftyp(), bx(b"moov", &mvhd)].concat();
        assert_eq!(run(&input, Container::Mp4), input);
    }

    fn nested_moov(levels: usize, inner: &[u8]) -> Vec<u8> {
        (0..levels).fold(inner.to_vec(), |inside, _| bx(b"moov", &inside))
    }

    #[test]
    fn nesting_past_the_depth_bound_is_left_alone_and_nesting_within_it_is_scrubbed() {
        let inner = bx(b"udta", GPS);
        let deep = [ftyp(), nested_moov(usize::from(MAX_DEPTH) + 5, &inner)].concat();
        assert_eq!(run(&deep, Container::Mp4), deep);

        let shallow = [ftyp(), nested_moov(4, &inner)].concat();
        assert!(!contains(&run(&shallow, Container::Mp4), GPS));
    }

    #[test]
    fn more_boxes_than_the_bound_are_left_alone() {
        let filler = bx(b"skip", &[]);
        let mut input = ftyp();
        for _ in 0..MAX_NODES + 10 {
            input.extend(&filler);
        }
        input.extend(bx(b"udta", GPS));
        assert_eq!(run(&input, Container::Mp4), input);

        let mut input = ftyp();
        for _ in 0..MAX_NODES - 10 {
            input.extend(&filler);
        }
        input.extend(bx(b"udta", GPS));
        assert!(
            !contains(&run(&input, Container::Mp4), GPS),
            "under the bound"
        );
    }

    #[test]
    fn noise_after_a_valid_ftyp_never_panics_or_changes_the_length() {
        for seed in 1..=300 {
            let input = [ftyp(), noise(seed, 4096)].concat();
            run(&input, Container::Mp4);
            run(&input, Container::Mov);
        }
    }

    #[test]
    fn a_damaged_valid_file_never_panics_or_changes_the_length() {
        let valid = sample(Layout::Wide, 1, true);
        for seed in 1..=500_u64 {
            let mut input = valid.clone();
            for hit in noise(seed, 8).chunks(2) {
                let at = usize::from(u16::from_be_bytes([hit[0], hit[1]])) % input.len();
                input[at] ^= hit[1] | 1;
            }
            run(&input, Container::Mp4);
        }
    }

    #[test]
    fn an_empty_file_and_a_bare_ftyp_are_fine() {
        assert!(run(&[], Container::Mp4).is_empty());
        let bare = ftyp();
        assert_eq!(run(&bare, Container::Mp4), bare);
    }

    // ---- WebM ----

    const EBML: u32 = 0x1a45_dfa3;
    const TRACKS: u32 = 0x1654_ae6b;
    const CLUSTER: u32 = 0x1f43_b675;
    const TITLE: u32 = 0x7ba9;
    const DATE_UTC: u32 = 0x4461;
    const MUXING_APP: u32 = 0x4d80;
    const WRITING_APP: u32 = 0x5741;
    const TIMECODE_SCALE: u32 = 0x2a_d7b1;

    const TITLE_TEXT: &[u8] = b"Holiday GPS 52.52N";
    const MUXER: &[u8] = b"libwebm-secret";
    const WRITER: &[u8] = b"SecretCam 9";
    const DATE: [u8; 8] = [0xd3, 0xa1, 0x5c, 0x77, 0x0e, 0x92, 0x41, 0x6b];
    const TAG_TEXT: &[u8] = b"GPS 52.5200N 13.4050E";
    const BLOCKS: &[u8] = b"CLUSTERDATA-0123456789";

    fn id_bytes(id: u32) -> Vec<u8> {
        let bytes = id.to_be_bytes();
        let skip = bytes.iter().take_while(|byte| **byte == 0).count();
        bytes[skip..].to_vec()
    }

    /// `value` as a size field `len` bytes long.
    fn vint(value: u64, len: usize) -> Vec<u8> {
        (value | (1 << (7 * len))).to_be_bytes()[8 - len..].to_vec()
    }

    /// An element with the shortest size field that fits.
    fn el(id: u32, payload: &[u8]) -> Vec<u8> {
        let value = payload.len() as u64;
        let len = (1..=8)
            .find(|len| value < (1_u64 << (7 * len)) - 1)
            .expect("fits");
        [id_bytes(id), vint(value, len), payload.to_vec()].concat()
    }

    /// An element whose size field takes 8 bytes.
    fn el_long(id: u32, payload: &[u8]) -> Vec<u8> {
        [
            id_bytes(id),
            vint(payload.len() as u64, 8),
            payload.to_vec(),
        ]
        .concat()
    }

    /// An element of unknown size.
    fn el_unknown(id: u32, payload: &[u8]) -> Vec<u8> {
        let unknown = vec![0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        [id_bytes(id), unknown, payload.to_vec()].concat()
    }

    fn ebml_header() -> Vec<u8> {
        el(EBML, &el(0x4282, b"webm"))
    }

    fn info() -> Vec<u8> {
        el(
            INFO,
            &[
                el(TIMECODE_SCALE, &[0x0f, 0x42, 0x40]),
                el(TITLE, TITLE_TEXT),
                el(DATE_UTC, &DATE),
                el(MUXING_APP, MUXER),
                el(WRITING_APP, WRITER),
            ]
            .concat(),
        )
    }

    /// Tags with a text longer than a one-byte size field holds.
    fn tags() -> Vec<u8> {
        let text = [TAG_TEXT, &[b' '; 200]].concat();
        let simple = [el(0x45a3, b"GPS"), el(0x4487, &text)].concat();
        el(TAGS, &el(0x7373, &el(0x67c8, &simple)))
    }

    #[derive(Clone, Copy)]
    enum Segment {
        Known,
        Wide,
        Unknown,
    }

    fn webm(segment: Segment) -> Vec<u8> {
        let cluster = el(CLUSTER, &[el(0xe7, &[0]), el(0xa3, BLOCKS)].concat());
        let body = [
            info(),
            el(TRACKS, b"TRACKS-DATA"),
            tags(),
            cluster.clone(),
            cluster,
            tags(),
        ]
        .concat();
        let segment = match segment {
            Segment::Known => el(SEGMENT, &body),
            Segment::Wide => el_long(SEGMENT, &body),
            Segment::Unknown => el_unknown(SEGMENT, &body),
        };
        [ebml_header(), segment].concat()
    }

    /// An element header read the plain way: (id, header length, size unless unknown).
    fn head_at(bytes: &[u8], pos: usize) -> (u32, usize, Option<usize>) {
        let id_len = bytes[pos].leading_zeros() as usize + 1;
        let id = bytes[pos..pos + id_len]
            .iter()
            .fold(0, |id, byte| (id << 8) | u32::from(*byte));
        let at = pos + id_len;
        let size_len = bytes[at].leading_zeros() as usize + 1;
        let mut size = u64::from(bytes[at]) & ((1 << (8 - size_len)) - 1);
        for byte in &bytes[at + 1..at + size_len] {
            size = (size << 8) | u64::from(*byte);
        }
        let unknown = size == (1 << (7 * size_len)) - 1;
        (
            id,
            id_len + size_len,
            (!unknown).then(|| usize::try_from(size).expect("fits")),
        )
    }

    type Flat = Vec<(usize, u32, usize, usize)>;

    /// (depth, id, offset, length) of every element of a valid file, down through `Segment` and
    /// `Info`.
    fn ebml_flat(bytes: &[u8]) -> Flat {
        fn walk(bytes: &[u8], start: usize, end: usize, depth: usize, out: &mut Flat) {
            let mut pos = start;
            while pos < end {
                let (id, header, size) = head_at(bytes, pos);
                let total = size.map_or(end - pos, |size| header + size);
                assert!(pos + total <= end, "an element at {pos}");
                out.push((depth, id, pos, total));
                if id == SEGMENT || id == INFO {
                    walk(bytes, pos + header, pos + total, depth + 1, out);
                }
                pos += total;
            }
        }
        let mut out = Vec::new();
        walk(bytes, 0, bytes.len(), 0, &mut out);
        out
    }

    #[test]
    fn a_webm_loses_its_tags_title_date_and_tool_names_and_keeps_its_clusters() {
        let secrets: [&[u8]; 6] = [TITLE_TEXT, MUXER, WRITER, &DATE, TAG_TEXT, b"GPS"];
        for segment in [Segment::Known, Segment::Wide, Segment::Unknown] {
            let input = webm(segment);
            for secret in secrets {
                assert!(contains(&input, secret), "the fixture holds {secret:?}");
            }
            let out = run(&input, Container::Webm);
            for secret in secrets {
                assert!(
                    !contains(&out, secret),
                    "{:?} survived",
                    String::from_utf8_lossy(secret)
                );
            }

            let before = ebml_flat(&input);
            let after = ebml_flat(&out);
            assert_eq!(before.len(), after.len());
            let mut voided = 0;
            for (old, new) in before.iter().zip(&after) {
                let (depth, id, offset, total) = *old;
                assert_eq!((depth, offset, total), (new.0, new.2, new.3), "same layout");
                let scrubbed = id == TAGS || (depth == 2 && INFO_METADATA.contains(&id));
                if scrubbed {
                    voided += 1;
                    assert_eq!(new.1, u32::from(VOID));
                    let (_, header, _) = head_at(&out, offset);
                    assert!(out[offset + header..offset + total].iter().all(|b| *b == 0));
                } else {
                    assert_eq!(new.1, id);
                    if id != SEGMENT && id != INFO {
                        let whole = offset..offset + total;
                        assert_eq!(out[whole.clone()], input[whole], "element {id:#x}");
                    }
                }
            }
            assert_eq!(voided, 6, "two Tags and four Info children");
            assert_eq!(after.iter().filter(|item| item.1 == CLUSTER).count(), 2);
            assert!(contains(&out, BLOCKS));
            assert!(contains(&out, b"TRACKS-DATA"));
        }
    }

    #[test]
    fn a_void_fills_any_element_of_two_bytes_or_more_exactly() {
        for total in [
            2_u64, 3, 4, 127, 128, 129, 130, 16_385, 16_386, 16_387, 16_388, 20_000,
        ] {
            let end = total as usize + 2;
            let mut file = Cursor::new(vec![0xff; end + 3]);
            Scrubber {
                file: &mut file,
                nodes: 0,
            }
            .void(2, total)
            .expect("void");
            let bytes = file.into_inner();
            assert_eq!(bytes[..2], [0xff, 0xff]);
            assert_eq!(bytes[end..], [0xff; 3], "past the element");
            let (id, header, size) = head_at(&bytes[2..], 0);
            assert_eq!(id, u32::from(VOID), "total {total}");
            assert_eq!(
                header + size.expect("known"),
                total as usize,
                "total {total}"
            );
            assert!(bytes[2 + header..end].iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn an_element_with_no_room_for_a_void_is_left_alone() {
        let mut file = Cursor::new(vec![0xff; 4]);
        Scrubber {
            file: &mut file,
            nodes: 0,
        }
        .void(1, 1)
        .expect("void");
        assert_eq!(file.into_inner(), [0xff; 4]);
    }

    #[test]
    fn empty_info_children_become_valid_voids() {
        let children = [el(TITLE, &[]), el(WRITING_APP, &[]), el(MUXING_APP, b"x")].concat();
        let input = [ebml_header(), el(SEGMENT, &el(INFO, &children))].concat();
        let out = run(&input, Container::Webm);
        let ids: Vec<_> = ebml_flat(&out).iter().map(|item| item.1).collect();
        let void = u32::from(VOID);
        assert_eq!(ids, [EBML, SEGMENT, INFO, void, void, void]);
    }

    #[test]
    fn headers_are_parsed_by_the_ebml_rules() {
        let head = |bytes: &[u8]| Element::parse(bytes).ok();
        let element = |id, header, size| Some(Element { id, header, size });
        assert_eq!(
            head(&[0x1a, 0x45, 0xdf, 0xa3, 0x9f]),
            element(EBML, 5, Some(31))
        );
        assert_eq!(head(&[0xec, 0x40, 0x80, 1]), element(0xec, 3, Some(128)));
        assert_eq!(
            head(&[0xa3, 0x01, 0, 0, 0, 0, 0, 0, 9]),
            element(0xa3, 9, Some(9))
        );
        assert_eq!(
            head(&[0x18, 0x53, 0x80, 0x67, 1, 255, 255, 255, 255, 255, 255, 255]),
            element(SEGMENT, 12, None)
        );
        assert_eq!(head(&[0xa3, 0xff]), element(0xa3, 2, None));
        for bad in [
            &[][..],
            &[0x00, 0x80],
            &[0x08, 1, 2, 3, 4, 0x80],
            &[0x1a, 0x45],
            &[0x1a, 0x45, 0xdf, 0xa3],
            &[0x1a, 0x45, 0xdf, 0xa3, 0x01, 0, 0],
            &[0xa3, 0x00],
        ] {
            assert!(head(bad).is_none(), "{bad:?}");
        }
    }

    /// Scrubs an `Info` with a title, then `bad` as the rest of an unknown-size `Segment`:
    /// everything from `bad` on must be as it was.
    fn assert_webm_stops_at_bad(bad: &[u8]) {
        let body = [info(), bad.to_vec()].concat();
        let input = [ebml_header(), el_unknown(SEGMENT, &body)].concat();
        let at = input.len() - bad.len();
        let out = run(&input, Container::Webm);
        assert!(
            !contains(&out[..at], TITLE_TEXT),
            "the part before the damage is scrubbed"
        );
        assert_eq!(out[at..], input[at..], "the rest is untouched");
    }

    #[test]
    fn webm_damage_stops_the_scrub() {
        let tags = tags();
        let overrun = [
            &id_bytes(INFO)[..],
            &[0x88],
            &el(TITLE, &[b'x'; 100]),
            &tags,
        ]
        .concat();
        let cases = [
            // A size past the end of the file.
            [&[0x12, 0x54, 0xc3, 0x67, 0x4f, 0xff][..], TAG_TEXT].concat(),
            // A cluster of unknown size.
            [el_unknown(CLUSTER, BLOCKS), tags.clone()].concat(),
            // An ID that starts with a zero byte, and one that is five bytes long.
            [&[0x00, 0x81, 0x00][..], &tags].concat(),
            [&[0x08, 1, 2, 3, 4, 0x81, 0][..], &tags].concat(),
            // A header cut short.
            vec![0x12, 0x54],
            // A child that overruns its `Info`.
            overrun,
            // A size field cut short.
            [&id_bytes(TAGS)[..], &[0x01, 0, 0, 0]].concat(),
        ];
        for bad in cases {
            assert_webm_stops_at_bad(&bad);
        }
    }

    #[test]
    fn a_top_level_element_of_unknown_size_other_than_the_segment_stops_the_scrub() {
        let input = [el_unknown(EBML, b"abc"), el(SEGMENT, &info())].concat();
        assert_eq!(run(&input, Container::Webm), input);
    }

    #[test]
    fn a_segment_cut_short_is_still_scrubbed() {
        let whole = webm(Segment::Known);
        let out = run(&whole[..whole.len() - 40], Container::Webm);
        assert!(!contains(&out, TITLE_TEXT));
    }

    #[test]
    fn more_elements_than_the_bound_are_left_alone() {
        let mut body = Vec::new();
        for _ in 0..MAX_NODES + 10 {
            body.extend([VOID, 0x80]);
        }
        body.extend(tags());
        let input = [ebml_header(), el_unknown(SEGMENT, &body)].concat();
        assert_eq!(run(&input, Container::Webm), input);
    }

    #[test]
    fn noise_after_an_ebml_head_never_panics_or_changes_the_length() {
        for seed in 1..=300 {
            let input = [ebml_header(), noise(seed, 4096)].concat();
            run(&input, Container::Webm);
            let input = [vec![0x1a, 0x45, 0xdf, 0xa3], noise(seed, 4096)].concat();
            run(&input, Container::Webm);
        }
    }

    #[test]
    fn a_damaged_webm_never_panics_or_changes_the_length() {
        let valid = webm(Segment::Known);
        for seed in 1..=500_u64 {
            let mut input = valid.clone();
            for hit in noise(seed, 8).chunks(2) {
                let at = usize::from(u16::from_be_bytes([hit[0], hit[1]])) % input.len();
                input[at] ^= hit[1] | 1;
            }
            run(&input, Container::Webm);
        }
    }

    #[test]
    fn an_empty_webm_and_a_bare_header_are_fine() {
        assert!(run(&[], Container::Webm).is_empty());
        let bare = ebml_header();
        assert_eq!(run(&bare, Container::Webm), bare);
    }
}
