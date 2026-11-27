//! Settings read from environment variables once at startup.

use std::net::SocketAddr;
use std::num::{NonZeroU16, NonZeroU64};
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;
use url::Url;

use crate::secrets::SecretKey;
use crate::storage::Storage;

/// How log lines are rendered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable multi-line output for local development.
    #[default]
    Pretty,
    /// One JSON object per line for log collectors.
    Json,
}
// 1: two dataless variants fit in one byte; the 254 unused values are a niche.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<LogFormat>() == 1);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Option<LogFormat>>() == std::mem::size_of::<LogFormat>());

/// A bad environment variable. The message names the variable.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid {var}: {reason}")]
pub struct ConfigError {
    /// Name of the offending environment variable.
    pub var: &'static str,
    /// What is wrong with its value.
    pub reason: Box<str>,
}
// 32: &'static str (16) + Box<str> (16); the reason is immutable once built, so no capacity word.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<ConfigError>() == 32);
#[cfg(target_pointer_width = "64")]
const _: () =
    assert!(std::mem::size_of::<Option<ConfigError>>() == std::mem::size_of::<ConfigError>());

const DEFAULT_TIMEOUT_SECS: NonZeroU16 = NonZeroU16::new(30).expect("30 is not zero");
const DEFAULT_MAX_VIDEO_MB: NonZeroU16 = NonZeroU16::new(50).expect("50 is not zero");

/// Service settings, read once at startup.
///
/// | Variable | Default | Meaning |
/// |---|---|---|
/// | `BIND_ADDR` | `127.0.0.1:8080` | Socket address to listen on. |
/// | `REQUEST_TIMEOUT_SECS` | `30` | Per-request limit, 1..=3600. |
/// | `LOG_FORMAT` | `pretty` | `pretty` or `json`. |
/// | `RUST_LOG` | `info` | Log filter (`tracing` env-filter syntax). |
/// | `DATABASE_URL` | required | Postgres URL. |
/// | `PUBLIC_URL` | `http://127.0.0.1:8080` | Public origin; its host is the federation domain. |
/// | `DATA_DIR` | `./data` | Media files live under `DATA_DIR/media` when `STORAGE=local`. |
/// | `STORAGE` | `local` | `local` or `s3`: where media files are kept. |
/// | `S3_BUCKET` | required for `s3` | Bucket name. |
/// | `S3_REGION` | `us-east-1` | Signing region. |
/// | `S3_ENDPOINT` | unset | Endpoint URL of a non-AWS service (MinIO, R2, B2, Garage). |
/// | `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` | required for `s3` | The access key pair. |
/// | `S3_PATH_STYLE` | true when `S3_ENDPOINT` is set | Bucket in the URL path instead of the host name. |
/// | `S3_PREFIX` | unset | Key prefix inside the bucket. |
/// | `SECURE_COOKIES` | true when `PUBLIC_URL` is https | Adds `Secure` to cookies. |
/// | `TRUST_PROXY` | `false` | Read the client IP from the last `X-Forwarded-For` hop. |
/// | `ADMIN_HANDLE` | unset | Only this handle may become the first admin. |
/// | `MAX_VIDEO_MB` | `50` | Largest video upload in MiB. |
/// | `FEDERATION_ALLOW_HTTP` | true when `PUBLIC_URL` is http | Allow plain-http federation peers. |
/// | `FEDERATION_ALLOW_PRIVATE_NETS` | `false` | Let federation fetches reach loopback and private addresses. Development and tests only. |
/// | `SOURCE_URL` | unset | AGPL source link shown to users. |
/// | `MESSAGES_E2EE_REQUIRED` | `true` | Direct messages between local accounts must be end-to-end encrypted. `false` lets two local accounts exchange plaintext until both have a message key. |
/// | `SECRET_KEY` | unset | 32 bytes as 64 hex characters or standard base64. Encrypts the ActivityPub signing keys in the database (AES-256-GCM); plaintext rows are sealed at startup. Once set, keep it: without it the sealed keys cannot be read. |
#[allow(
    clippy::struct_excessive_bools,
    reason = "one independent on/off switch per environment variable"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Address to listen on.
    pub bind_addr: SocketAddr,
    /// Requests running longer than this many seconds get `408 Request Timeout`.
    pub request_timeout_secs: NonZeroU16,
    /// Log rendering.
    pub log_format: LogFormat,
    /// Postgres connection URL.
    pub database_url: Box<str>,
    /// Public origin, without a trailing slash: `https://example.com`.
    pub public_url: Box<str>,
    /// Root of locally stored files.
    pub data_dir: Box<Path>,
    /// Where media files are kept.
    pub storage: Storage,
    /// Mark cookies `Secure`.
    pub secure_cookies: bool,
    /// Trust the last `X-Forwarded-For` hop as the client IP.
    pub trust_proxy: bool,
    /// The only handle allowed to become the first admin, when set.
    pub admin_handle: Option<Box<str>>,
    /// Largest video upload in MiB.
    pub max_video_mb: NonZeroU16,
    /// Allow federation with plain-http peers (development only).
    pub federation_allow_http: bool,
    /// Let the federation client reach loopback and private addresses (development and tests
    /// only; never inferred from `PUBLIC_URL`).
    pub federation_allow_private_nets: bool,
    /// Where users can get the running source code (AGPL).
    pub source_url: Option<Box<str>>,
    /// Direct messages between local accounts must be sealed; `false` allows plaintext while one
    /// side has no message key.
    pub messages_e2ee_required: bool,
    /// Key that seals the ActivityPub signing keys at rest (`SECRET_KEY`); `None` stores them
    /// as plaintext.
    pub(crate) secret_key: Option<Box<SecretKey>>,
}
// 144: SocketAddr (32) + five boxed str/path fields (5 × 16) + Storage (8, a boxed S3 whose null is `Local`) + Option<Box<SecretKey>> (8, null niche) + two NonZeroU16 + LogFormat + five bools, padded to 8.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Config>() == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Option<Config>>() == std::mem::size_of::<Config>());

impl Config {
    /// The per-request limit as a [`Duration`].
    #[must_use]
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(NonZeroU64::from(self.request_timeout_secs).get())
    }

    /// Reads the settings from the process environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] naming the first variable that is missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|var| std::env::var(var).ok())
    }

    /// Reads the settings through `lookup`; unset variables return `None`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] naming the first variable that is missing or invalid.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |var: &str| lookup(var).filter(|raw| !raw.trim().is_empty());

        let database_url = match get("DATABASE_URL") {
            None => return Err(invalid("DATABASE_URL", "not set")),
            Some(raw) if raw.starts_with("postgres://") || raw.starts_with("postgresql://") => {
                raw.trim().into()
            }
            Some(_) => {
                return Err(invalid(
                    "DATABASE_URL",
                    "must start with postgres:// or postgresql://",
                ));
            }
        };

        let bind_addr = match get("BIND_ADDR") {
            None => SocketAddr::from(([127, 0, 0, 1], 8080)),
            Some(raw) => raw.trim().parse().map_err(|err| {
                invalid(
                    "BIND_ADDR",
                    &format!("{raw:?} is not a socket address like 0.0.0.0:8080 ({err})"),
                )
            })?,
        };

        let request_timeout_secs = match get("REQUEST_TIMEOUT_SECS") {
            None => DEFAULT_TIMEOUT_SECS,
            Some(raw) => parse_bounded("REQUEST_TIMEOUT_SECS", &raw, 3600)?,
        };

        let log_format = match get("LOG_FORMAT") {
            None => LogFormat::Pretty,
            Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
                "pretty" => LogFormat::Pretty,
                "json" => LogFormat::Json,
                _ => {
                    return Err(invalid(
                        "LOG_FORMAT",
                        &format!("{raw:?} is not `pretty` or `json`"),
                    ));
                }
            },
        };

        let public_url = get("PUBLIC_URL").unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
        let public_url = parse_origin(&public_url)?;
        let https = public_url.starts_with("https://");

        let max_video_mb = match get("MAX_VIDEO_MB") {
            None => DEFAULT_MAX_VIDEO_MB,
            Some(raw) => parse_bounded("MAX_VIDEO_MB", &raw, 1024)?,
        };

        Ok(Self {
            bind_addr,
            request_timeout_secs,
            log_format,
            database_url,
            public_url,
            data_dir: PathBuf::from(get("DATA_DIR").as_deref().unwrap_or("./data")).into(),
            storage: Storage::from_lookup(&get)?,
            secure_cookies: parse_bool("SECURE_COOKIES", get("SECURE_COOKIES"), https)?,
            trust_proxy: parse_bool("TRUST_PROXY", get("TRUST_PROXY"), false)?,
            admin_handle: get("ADMIN_HANDLE").map(|raw| raw.trim().into()),
            max_video_mb,
            federation_allow_http: parse_bool(
                "FEDERATION_ALLOW_HTTP",
                get("FEDERATION_ALLOW_HTTP"),
                !https,
            )?,
            federation_allow_private_nets: parse_bool(
                "FEDERATION_ALLOW_PRIVATE_NETS",
                get("FEDERATION_ALLOW_PRIVATE_NETS"),
                false,
            )?,
            source_url: get("SOURCE_URL").map(|raw| raw.trim().into()),
            messages_e2ee_required: parse_bool(
                "MESSAGES_E2EE_REQUIRED",
                get("MESSAGES_E2EE_REQUIRED"),
                true,
            )?,
            secret_key: get("SECRET_KEY")
                .map(|raw| {
                    SecretKey::parse(&raw).map(Box::new).ok_or_else(|| {
                        invalid(
                            "SECRET_KEY",
                            "must be 32 bytes as 64 hex characters or standard base64",
                        )
                    })
                })
                .transpose()?,
        })
    }

    /// Host (with a non-default port) of `PUBLIC_URL`: the federation domain.
    #[must_use]
    pub fn domain(&self) -> &str {
        self.public_url
            .split_once("://")
            .map_or(&*self.public_url, |(_, host)| host)
    }
}

pub(crate) fn invalid(var: &'static str, reason: &str) -> ConfigError {
    ConfigError {
        var,
        reason: reason.into(),
    }
}

fn parse_bounded(var: &'static str, raw: &str, max: u16) -> Result<NonZeroU16, ConfigError> {
    let value: NonZeroU16 = raw.trim().parse().map_err(|err| {
        invalid(
            var,
            &format!("{raw:?} is not a whole number in 1..={max} ({err})"),
        )
    })?;
    if value.get() > max {
        return Err(invalid(var, &format!("{value} is outside 1..={max}")));
    }
    Ok(value)
}

pub(crate) fn parse_bool(
    var: &'static str,
    raw: Option<String>,
    default: bool,
) -> Result<bool, ConfigError> {
    let Some(raw) = raw else {
        return Ok(default);
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(invalid(var, &format!("{raw:?} is not true or false"))),
    }
}

/// Parses `PUBLIC_URL` into a bare origin like `https://example.com:8443`.
fn parse_origin(raw: &str) -> Result<Box<str>, ConfigError> {
    let url = Url::parse(raw.trim())
        .map_err(|err| invalid("PUBLIC_URL", &format!("{raw:?} is not a URL ({err})")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(invalid("PUBLIC_URL", "must start with http:// or https://"));
    }
    if url.host_str().is_none() || url.path() != "/" || url.query().is_some() {
        return Err(invalid(
            "PUBLIC_URL",
            &format!("{raw:?} must be an origin like https://example.com, without a path"),
        ));
    }
    Ok(url.origin().ascii_serialization().into())
}

#[cfg(test)]
mod test {
    use super::*;

    fn config(vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
        Config::from_lookup(|name| {
            vars.iter()
                .chain(&[("DATABASE_URL", "postgres://localhost/tuita")])
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    fn database_url_is_required() {
        let err = Config::from_lookup(|_| None).expect_err("DATABASE_URL is required");
        assert_eq!(err.to_string(), "invalid DATABASE_URL: not set");
    }

    #[test]
    fn defaults_when_only_database_url_set() {
        assert_eq!(
            config(&[]),
            Ok(Config {
                bind_addr: SocketAddr::from(([127, 0, 0, 1], 8080)),
                request_timeout_secs: DEFAULT_TIMEOUT_SECS,
                log_format: LogFormat::Pretty,
                database_url: "postgres://localhost/tuita".into(),
                public_url: "http://127.0.0.1:8080".into(),
                data_dir: Path::new("./data").into(),
                storage: Storage::Local,
                secure_cookies: false,
                trust_proxy: false,
                admin_handle: None,
                max_video_mb: DEFAULT_MAX_VIDEO_MB,
                federation_allow_http: true,
                federation_allow_private_nets: false,
                source_url: None,
                messages_e2ee_required: true,
                secret_key: None,
            })
        );
    }

    #[test]
    fn reads_every_variable() {
        let parsed = config(&[
            ("BIND_ADDR", "0.0.0.0:9000"),
            ("REQUEST_TIMEOUT_SECS", "5"),
            ("LOG_FORMAT", "JSON"),
            ("PUBLIC_URL", "https://social.example:8443/"),
            ("DATA_DIR", "/var/lib/tuita"),
            ("TRUST_PROXY", "true"),
            ("ADMIN_HANDLE", "ada"),
            ("MAX_VIDEO_MB", "100"),
            ("SOURCE_URL", "https://git.example/tuita"),
            ("MESSAGES_E2EE_REQUIRED", "false"),
        ])
        .expect("valid config");
        assert_eq!(parsed.bind_addr, SocketAddr::from(([0, 0, 0, 0], 9000)));
        assert_eq!(parsed.request_timeout_secs.get(), 5);
        assert_eq!(parsed.log_format, LogFormat::Json);
        assert_eq!(&*parsed.public_url, "https://social.example:8443");
        assert_eq!(parsed.domain(), "social.example:8443");
        assert_eq!(&*parsed.data_dir, Path::new("/var/lib/tuita"));
        assert!(parsed.trust_proxy);
        assert_eq!(parsed.admin_handle.as_deref(), Some("ada"));
        assert_eq!(parsed.max_video_mb.get(), 100);
        assert_eq!(
            parsed.source_url.as_deref(),
            Some("https://git.example/tuita")
        );
        assert!(!parsed.messages_e2ee_required);
    }

    #[test]
    fn https_flips_cookie_and_federation_defaults() {
        let parsed = config(&[("PUBLIC_URL", "https://social.example")]).expect("valid config");
        assert!(parsed.secure_cookies);
        assert!(!parsed.federation_allow_http);
        assert_eq!(parsed.domain(), "social.example");

        let overridden = config(&[
            ("PUBLIC_URL", "https://social.example"),
            ("SECURE_COOKIES", "false"),
            ("FEDERATION_ALLOW_HTTP", "1"),
        ])
        .expect("valid config");
        assert!(!overridden.secure_cookies);
        assert!(overridden.federation_allow_http);
    }

    #[test]
    fn errors_name_the_variable() {
        for (var, value) in [
            ("DATABASE_URL", "mysql://x"),
            ("BIND_ADDR", "nope"),
            ("REQUEST_TIMEOUT_SECS", "0"),
            ("REQUEST_TIMEOUT_SECS", "abc"),
            ("REQUEST_TIMEOUT_SECS", "3601"),
            ("LOG_FORMAT", "xml"),
            ("PUBLIC_URL", "ftp://example.com"),
            ("PUBLIC_URL", "https://example.com/sub"),
            ("SECURE_COOKIES", "maybe"),
            ("MAX_VIDEO_MB", "0"),
        ] {
            let err = config(&[(var, value)]).expect_err("value must be rejected");
            assert_eq!(err.var, var, "{var}={value}");
            assert!(
                err.to_string().starts_with(&format!("invalid {var}:")),
                "{err}"
            );
        }
    }

    const S3_KEYS: [(&str, &str); 4] = [
        ("STORAGE", "s3"),
        ("S3_BUCKET", "tuita-media"),
        ("S3_ACCESS_KEY_ID", "AKIDEXAMPLE"),
        ("S3_SECRET_ACCESS_KEY", "s3cr3t-value"),
    ];

    /// A config with the S3 variables of `S3_KEYS`, then `extra`; later entries win and an
    /// empty value unsets the variable.
    fn s3_config(extra: &[(&str, &str)]) -> Result<Config, ConfigError> {
        Config::from_lookup(|name| {
            [("DATABASE_URL", "postgres://localhost/tuita")]
                .iter()
                .chain(&S3_KEYS)
                .chain(extra)
                .rfind(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    fn s3(config: Config) -> crate::storage::S3 {
        match config.storage {
            Storage::S3(s3) => *s3,
            Storage::Local => panic!("expected STORAGE=s3"),
        }
    }

    #[test]
    fn storage_is_local_unless_s3_is_asked_for() {
        assert_eq!(config(&[]).expect("defaults").storage, Storage::Local);
        let ignored = config(&[("STORAGE", " Local "), ("S3_BUCKET", "!!")]).expect("local");
        assert_eq!(
            ignored.storage,
            Storage::Local,
            "S3_* mean nothing for local"
        );
    }

    #[test]
    fn s3_defaults() {
        let s3 = s3(s3_config(&[]).expect("valid s3 settings"));
        assert_eq!(
            s3,
            crate::storage::S3 {
                bucket: "tuita-media".into(),
                region: "us-east-1".into(),
                endpoint: None,
                credentials: crate::storage::S3Credentials {
                    access_key_id: "AKIDEXAMPLE".into(),
                    secret_access_key: "s3cr3t-value".into(),
                },
                path_style: false,
                prefix: None,
            }
        );
    }

    #[test]
    fn a_custom_endpoint_turns_path_style_on_unless_told_otherwise() {
        let minio = s3(s3_config(&[("S3_ENDPOINT", "http://127.0.0.1:9000/")]).expect("minio"));
        assert_eq!(minio.endpoint.as_deref(), Some("http://127.0.0.1:9000"));
        assert!(minio.path_style);

        let garage = s3(s3_config(&[
            ("S3_ENDPOINT", "https://s3.example.com"),
            ("S3_PATH_STYLE", "false"),
            ("S3_REGION", "garage"),
            ("S3_PREFIX", "/media/prod/"),
        ])
        .expect("garage"));
        assert!(!garage.path_style);
        assert_eq!(&*garage.region, "garage");
        assert_eq!(garage.prefix.as_deref(), Some("media/prod"));
    }

    #[test]
    fn storage_errors_name_the_variable() {
        let cases: [(&str, &[(&str, &str)]); 15] = [
            ("STORAGE", &[("STORAGE", "ftp")]),
            ("S3_BUCKET", &[("S3_BUCKET", "")]),
            ("S3_BUCKET", &[("S3_BUCKET", "Has_Caps")]),
            ("S3_BUCKET", &[("S3_BUCKET", "ab")]),
            ("S3_BUCKET", &[("S3_BUCKET", "-leading")]),
            ("S3_REGION", &[("S3_REGION", "US East")]),
            ("S3_ENDPOINT", &[("S3_ENDPOINT", "minio:9000")]),
            ("S3_ENDPOINT", &[("S3_ENDPOINT", "ftp://minio")]),
            ("S3_ENDPOINT", &[("S3_ENDPOINT", "http://user:pw@minio")]),
            ("S3_ENDPOINT", &[("S3_ENDPOINT", "http://minio/?x=1")]),
            ("S3_PATH_STYLE", &[("S3_PATH_STYLE", "maybe")]),
            ("S3_PREFIX", &[("S3_PREFIX", "a/../b")]),
            ("S3_ACCESS_KEY_ID", &[("S3_ACCESS_KEY_ID", "")]),
            ("S3_SECRET_ACCESS_KEY", &[("S3_SECRET_ACCESS_KEY", "")]),
            (
                "S3_SECRET_ACCESS_KEY",
                &[("S3_SECRET_ACCESS_KEY", "bad\u{7}key")],
            ),
        ];
        for (var, extra) in cases {
            let err = s3_config(extra).expect_err("value must be rejected");
            assert_eq!(err.var, var, "{extra:?}");
            assert!(
                err.to_string().starts_with(&format!("invalid {var}:")),
                "{err}"
            );
        }
    }

    #[test]
    fn secrets_are_redacted_in_debug_and_errors() {
        let shown = format!("{:?}", s3_config(&[]).expect("valid s3 settings"));
        assert!(!shown.contains("s3cr3t-value"), "{shown}");
        assert!(!shown.contains("AKIDEXAMPLE"), "{shown}");
        assert!(shown.contains("tuita-media"), "{shown}");

        let err = s3_config(&[("S3_SECRET_ACCESS_KEY", "bad\u{7}s3cr3t")]).expect_err("control");
        assert!(!err.to_string().contains("s3cr3t"), "{err}");
    }

    const KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    #[test]
    fn secret_key_reads_hex_and_base64() {
        let hex = config(&[("SECRET_KEY", KEY_HEX)]).expect("hex key");
        assert!(hex.secret_key.is_some());
        let b64 = config(&[("SECRET_KEY", "ABEiM0RVZneImaq7zN3u/wARIjNEVWZ3iJmqu8zd7v8=")])
            .expect("base64 key");
        assert_eq!(
            b64.secret_key, hex.secret_key,
            "the same 32 bytes in both spellings"
        );
        assert_eq!(config(&[]).expect("defaults").secret_key, None);
    }

    #[test]
    fn secret_key_rejects_other_lengths_without_echoing_the_value() {
        for bad in [
            "abcd",
            &KEY_HEX[..62],
            &format!("{KEY_HEX}00"),
            "not a key at all",
        ] {
            let err = config(&[("SECRET_KEY", bad)]).expect_err("bad key");
            assert_eq!(err.var, "SECRET_KEY", "{bad}");
            assert!(err.to_string().starts_with("invalid SECRET_KEY:"), "{err}");
            assert!(!err.to_string().contains(bad), "{err}");
        }
    }

    #[test]
    fn secret_key_is_redacted_in_debug() {
        let shown = format!("{:?}", config(&[("SECRET_KEY", KEY_HEX)]).expect("hex key"));
        assert!(!shown.contains(KEY_HEX), "{shown}");
        assert!(!shown.contains("0011"), "{shown}");
        assert!(shown.contains("SecretKey(<redacted>)"), "{shown}");
    }
}
