//! Where media files live: the settings behind `STORAGE`.

use std::fmt;

use object_store::path::Path as ObjectPath;
use url::Url;

use crate::config::{ConfigError, invalid, parse_bool};

/// Where uploaded and federated media files are kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Storage {
    /// Files under `DATA_DIR/media`.
    Local,
    /// Objects in an S3-compatible bucket.
    S3(Box<S3>),
}
// 8: the S3 settings are boxed, so the null pointer is the `Local` niche.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Storage>() == 8);

/// An S3-compatible bucket and how to reach it (AWS S3, MinIO, Cloudflare R2, Backblaze B2, Garage).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct S3 {
    /// Bucket name (`S3_BUCKET`).
    pub bucket: Box<str>,
    /// Signing region (`S3_REGION`, default `us-east-1`).
    pub region: Box<str>,
    /// Endpoint URL of a non-AWS service (`S3_ENDPOINT`), without a trailing slash.
    pub endpoint: Option<Box<str>>,
    /// Access key pair (`S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY`).
    pub credentials: S3Credentials,
    /// Put the bucket in the URL path instead of the host name (`S3_PATH_STYLE`, default true
    /// when `S3_ENDPOINT` is set).
    pub path_style: bool,
    /// Key prefix inside the bucket (`S3_PREFIX`), without leading or trailing slashes.
    pub prefix: Option<Box<str>>,
}

/// An access key pair. [`fmt::Debug`] prints neither half.
#[derive(Clone, PartialEq, Eq)]
pub struct S3Credentials {
    /// `S3_ACCESS_KEY_ID`.
    pub access_key_id: Box<str>,
    /// `S3_SECRET_ACCESS_KEY`.
    pub secret_access_key: Box<str>,
}

impl fmt::Debug for S3Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("S3Credentials(<redacted>)")
    }
}

const DEFAULT_REGION: &str = "us-east-1";

impl Storage {
    /// Reads `STORAGE` and, for `s3`, the `S3_*` variables. `get` returns a variable's trimmed,
    /// non-blank value.
    pub(crate) fn from_lookup(get: &impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        match get("STORAGE")
            .map(|raw| raw.trim().to_ascii_lowercase())
            .as_deref()
        {
            None | Some("local") => Ok(Self::Local),
            Some("s3") => S3::from_lookup(get).map(|s3| Self::S3(Box::new(s3))),
            Some(other) => Err(invalid(
                "STORAGE",
                &format!("{other:?} is not `local` or `s3`"),
            )),
        }
    }
}

impl S3 {
    fn from_lookup(get: &impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let bucket = get("S3_BUCKET").ok_or_else(|| invalid("S3_BUCKET", "not set"))?;
        let bucket = bucket.trim();
        check_bucket(bucket)?;

        let region = get("S3_REGION").unwrap_or_else(|| DEFAULT_REGION.to_owned());
        let region = region.trim();
        if !region
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(invalid(
                "S3_REGION",
                &format!("{region:?} must be lowercase letters, digits and dashes, like us-east-1"),
            ));
        }

        let endpoint = get("S3_ENDPOINT")
            .map(|raw| parse_endpoint(&raw))
            .transpose()?;

        let credentials = S3Credentials {
            access_key_id: require_key(get, "S3_ACCESS_KEY_ID")?,
            secret_access_key: require_key(get, "S3_SECRET_ACCESS_KEY")?,
        };

        let path_style = parse_bool("S3_PATH_STYLE", get("S3_PATH_STYLE"), endpoint.is_some())?;

        let prefix = get("S3_PREFIX")
            .map(|raw| parse_prefix(&raw))
            .transpose()?
            .flatten();

        Ok(Self {
            bucket: bucket.into(),
            region: region.into(),
            endpoint,
            credentials,
            path_style,
            prefix,
        })
    }
}

/// The access key variables hold secrets, so an error never quotes their value.
fn require_key(
    get: &impl Fn(&str) -> Option<String>,
    var: &'static str,
) -> Result<Box<str>, ConfigError> {
    let value = get(var).ok_or_else(|| invalid(var, "not set (both S3 keys are required)"))?;
    let value = value.trim();
    if value.chars().any(char::is_control) {
        return Err(invalid(var, "contains a control character"));
    }
    Ok(value.into())
}

/// S3 bucket naming rules: 3 to 63 lowercase letters, digits, dots and dashes, starting and
/// ending with a letter or digit. Every S3-compatible service accepts these names.
fn check_bucket(bucket: &str) -> Result<(), ConfigError> {
    let edge = |b: Option<&u8>| b.is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
    let bytes = bucket.as_bytes();
    let charset = bytes
        .iter()
        .all(|&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-');
    if (3..=63).contains(&bytes.len()) && charset && edge(bytes.first()) && edge(bytes.last()) {
        Ok(())
    } else {
        Err(invalid(
            "S3_BUCKET",
            &format!(
                "{bucket:?} must be 3 to 63 lowercase letters, digits, dots or dashes, starting and ending with a letter or digit"
            ),
        ))
    }
}

fn parse_endpoint(raw: &str) -> Result<Box<str>, ConfigError> {
    let url = Url::parse(raw.trim())
        .map_err(|err| invalid("S3_ENDPOINT", &format!("{raw:?} is not a URL ({err})")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(invalid(
            "S3_ENDPOINT",
            "must start with http:// or https://",
        ));
    }
    if url.host_str().is_none() {
        return Err(invalid("S3_ENDPOINT", &format!("{raw:?} has no host")));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(invalid(
            "S3_ENDPOINT",
            "must not carry credentials; use S3_ACCESS_KEY_ID and S3_SECRET_ACCESS_KEY",
        ));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(invalid(
            "S3_ENDPOINT",
            &format!("{raw:?} must not have a query or fragment"),
        ));
    }
    Ok(url.as_str().trim_end_matches('/').into())
}

/// `None` for a prefix that is only slashes.
fn parse_prefix(raw: &str) -> Result<Option<Box<str>>, ConfigError> {
    let prefix = raw.trim().trim_matches('/');
    if prefix.is_empty() {
        return Ok(None);
    }
    ObjectPath::parse(prefix).map_err(|err| {
        invalid(
            "S3_PREFIX",
            &format!("{raw:?} is not a usable key prefix ({err})"),
        )
    })?;
    Ok(Some(prefix.into()))
}
