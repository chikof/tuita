---
name: rust-layout
description: Load when defining or changing a struct, enum, error type or collection element type, when a layout or size lint fires, or when adding or changing a size_of pin.
---

# Rust layout and niches

Goal: types no bigger than they must be, with `Option<T>` costing nothing extra where possible. Measure; do not guess.

## Measure first

```sh
scripts/type-sizes.sh              # this workspace's own types, largest first
scripts/type-sizes.sh Token        # types whose name contains Token (wrappers included)
scripts/type-sizes.sh 'Option<'    # every Option around a workspace type
```

Output lists size, alignment, each field with offset, padding, variants, and niche info. It uses `RUSTC_BOOTSTRAP=1 -Zprint-type-sizes`: local only, format unstable, never part of `check`. It cleans workspace crates in `target/type-sizes` first.

Also available as `bacon type-sizes`.

## Niches

A niche is a bit pattern a type never uses. `Option<T>` stores `None` there, so no extra tag.

| Type | Niche | Result |
|---|---|---|
| `&T`, `Box<T>`, `Rc<T>`, `Arc<T>`, `NonNull<T>` | null | `Option<Box<T>>` = `Box<T>` |
| `NonZeroU8..U128`, `NonZeroI*`, `NonZeroUsize` | 0 | `Option<NonZeroU32>` = 4 bytes, `Option<u32>` = 8 |
| `bool` | 2..=255 | `Option<bool>` = 1 byte |
| `char` | above `0x10FFFF` | `Option<char>` = 4 bytes |
| `String`, `Vec<T>` | capacity above `isize::MAX` | `Option<String>` = 24 bytes |
| dataless enum | unused discriminants | `Option<Mode>` = `Mode` |

Rules of thumb:

- `Option<u32>` field that is never 0: use `Option<NonZeroU32>`. Same for ids and counts (`NonZeroU16` timeouts, as the service template does).
- Nested options: `Option<Option<T>>` is a smell (`option_option`). Use a three-state enum.
- Do not `Box<T>` a small `T` for "niche"; references and boxes already have one.
- Every `u8`/`u16`/`u32`/`i*`/`f*` accepts all patterns: no niche. Wrap in a newtype with `NonZero*` or an enum if you need one.

## Enums

Size = tag + largest variant, rounded to alignment. Rust fills a niche of the largest variant when it can, but one fat variant makes every value fat.

- Largest variant over 128 bytes: box the payload (`large_enum_variant`).
- One variant much bigger than the rest: box that one (`variant_size_differences`).
- Prefer `Box<str>` / `Box<[T]>` for text and slices that are built once and never grown: 16 bytes against 24 for `String` / `Vec<T>`, no capacity word.
- `Vec<T>` and `String` remain right for anything that grows. Do not wrap them as `Box<Vec<T>>` (`box_collection`).
- `Rc<str>` / `Arc<str>`, not `Rc<String>` (`rc_buffer`). `Rc<RefCell<T>>`, not `Rc<Mutex<T>>` (`rc_mutex`).
- Errors: keep `Result<_, E>` small. `E` over 128 bytes: box it (`result_large_err`, on by default).
- Futures over 4 KiB: `Box::pin` them before `.await` or spawn (`large_futures`).
- Big arrays go on the heap, not the stack (`large_stack_arrays`, `large_const_arrays`).

Field order does not matter for `repr(Rust)`: rustc reorders to minimise padding. It matters for `repr(C)` and when you pin a size, so list fields large-to-small there.

## Size pins

Pin hot types: elements of big collections, values passed by value in hot paths, error types, small `Copy` types. Do not pin every type.

```rust
// 16: a Box<str> (pointer + length), no capacity word; the null pointer is a niche.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Greeting>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Option<Greeting>>() == std::mem::size_of::<Greeting>());
```

- The comment says why the number is what it is. A bare number invites a blind bump.
- Pin `Option<T>` equal to `T` when the niche is the point.
- A pin failing means the layout changed. Run `scripts/type-sizes.sh <Type>`, see which field grew, decide. Raising a number needs a line in the commit body saying why.
- Pins use `target_pointer_width = "64"`; other widths are not checked.

## What is gated and what is review-only

`check` runs clippy with `-D warnings`, so each lint below is a hard failure, with thresholds from `clippy.toml`.

| Gated by lint | Threshold (`clippy.toml`) |
|---|---|
| `large_enum_variant` | `enum-variant-size-threshold = 128` |
| `large_futures` | `future-size-threshold = 4096` |
| `trivially_copy_pass_by_ref` | `trivial-copy-size-limit = 16` |
| `result_large_err` (clippy default on) | `large-error-threshold = 128` |
| `variant_size_differences` (rustc lint) | built in |
| `large_stack_frames`, `large_stack_arrays`, `large_const_arrays`, `large_types_passed_by_value`, `boxed_local`, `unnecessary_box_returns` | clippy defaults |
| `box_collection`, `vec_box`, `redundant_allocation`, `rc_buffer`, `rc_mutex`, `option_option`, `ref_option`, `ref_option_ref`, `zero_sized_map_values`, `inefficient_to_string`, `non_zero_suggestions`, `needless_pass_by_value` | none |

Size pins are gated too: a wrong `const` assert fails the build.

`avoid-breaking-exported-api`:

- Application kinds (cli, service, workspace) set it `false`: the lints above also apply to `pub` signatures.
- Libraries keep clippy's default (`true`): lints that would change a public signature do not fire on `pub` items. Affected here: `option_option`, `rc_buffer`, `rc_mutex`, `box_collection`, `vec_box`, `redundant_allocation`, `unnecessary_box_returns`, `ref_option`, `trivially_copy_pass_by_ref`, `large_types_passed_by_value`. Review those by hand, since a fix is a breaking change. Private code is still linted.

Review-only (nothing fails, you decide):

- Choosing `Box<str>` over `String`, `Box<[T]>` over `Vec<T>` for frozen data.
- Using `NonZero*` instead of `Option<u32>` with a "0 is invalid" convention.
- Whether a type deserves a size pin.
- Boxing a variant that is under 128 bytes but rare and large relative to its siblings.
- Index types (`u32` over `usize`) in big tables.
- Layout of `pub` items in libraries (see above).

## Workflow for a layout change

1. `scripts/type-sizes.sh <Type>` before. Note the size.
2. Change the type. Fix compile errors (skill `rust-change`).
3. `scripts/type-sizes.sh <Type>` after. Note the size.
4. Update or add the pin with its why-comment.
5. `check` (skill `verify`).
