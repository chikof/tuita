//! The object store behind every media file: local disk, an S3-compatible bucket, or memory.
//!
//! A file's key is the `path` column of its `media` row (`{id}.{ext}`). The server only ever
//! writes a whole object, reads it or a byte range of it, and deletes it, so the store is a
//! handful of methods over `object_store`.

use std::fmt;
use std::io;
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::body::Bytes;
use futures_util::StreamExt as _;
use futures_util::stream::BoxStream;
use object_store::aws::AmazonS3Builder;
use object_store::buffered::BufWriter;
use object_store::client::{HttpClient, HttpConnector};
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;
use object_store::path::Path as ObjectPath;
use object_store::prefix::PrefixStore;
use object_store::{
    ClientOptions, GetOptions, GetRange, ObjectStore, ObjectStoreExt as _, PutPayload, RetryConfig,
};
use thiserror::Error;
use tokio::io::AsyncWriteExt as _;

use crate::config::{Config, ConfigError, invalid};
use crate::net::tls_config;
use crate::storage::{S3, Storage};

/// Longest to wait for a connection to the store.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Longest a connection to the store may stay silent. The whole transfer has no limit of its
/// own: the request timeout of the upload route bounds it.
const READ_TIMEOUT: Duration = Duration::from_secs(60);
/// A failed request is retried at most this many times, within [`RETRY_WINDOW`].
const MAX_RETRIES: usize = 3;
const RETRY_WINDOW: Duration = Duration::from_secs(30);
/// The object the startup check writes and deletes.
const PROBE_KEY: &str = ".tuita-startup-check";

/// A store operation that failed.
#[derive(Debug, Error)]
pub enum StoreError {
    /// There is no object under the key.
    #[error("no object {0:?}")]
    NotFound(Box<str>),
    /// The key is not a valid object path.
    #[error("{0:?} is not a valid object key")]
    BadKey(Box<str>),
    /// The backend refused or failed the request.
    #[error("object store request failed")]
    Backend(#[source] Box<object_store::Error>),
    /// Reading the local temp file or writing to the store failed.
    #[error("object store transfer failed")]
    Io(#[from] io::Error),
}
// 24: a tag plus the widest payload, a boxed str (16).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<StoreError>() == 24);

impl StoreError {
    fn backend(key: &str, err: object_store::Error) -> Self {
        match err {
            object_store::Error::NotFound { .. } => Self::NotFound(key.into()),
            other => Self::Backend(Box::new(other)),
        }
    }
}

/// What the store knows about an object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectMeta {
    /// Size in bytes.
    pub size: u64,
    /// When the object was last written.
    pub modified: SystemTime,
    /// The store's entity tag, quoted for use as an HTTP `ETag`.
    pub etag: Option<Box<str>>,
}

/// An object, or a byte range of it, being read.
pub struct Object {
    /// The whole object's metadata.
    pub meta: ObjectMeta,
    /// The bytes `body` yields, as offsets into the object.
    pub range: Range<u64>,
    /// The content in order.
    pub body: BoxStream<'static, io::Result<Bytes>>,
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Object")
            .field("meta", &self.meta)
            .field("range", &self.range)
            .finish_non_exhaustive()
    }
}

/// Where media files live. Cheap to clone.
#[derive(Clone)]
pub struct MediaStore(Arc<dyn ObjectStore>);

impl fmt::Debug for MediaStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MediaStore({})", self.0)
    }
}

impl MediaStore {
    /// The store `config` asks for, and for S3 a check that the bucket answers and accepts a
    /// write and a delete.
    ///
    /// # Errors
    ///
    /// A [`ConfigError`] naming the variable at fault (`invalid S3_BUCKET: ...`).
    pub async fn open(config: &Config) -> Result<Self, ConfigError> {
        let store = Self::from_config(config)?;
        if let Storage::S3(s3) = &config.storage {
            store.probe().await.map_err(|err| probe_failure(s3, &err))?;
        }
        Ok(store)
    }

    /// The store `config` asks for, without contacting it.
    ///
    /// # Errors
    ///
    /// A [`ConfigError`] when the S3 settings cannot make a client, or `DATA_DIR` is not an
    /// absolute-able path.
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        match &config.storage {
            Storage::Local => Self::local(&config.data_dir.join("media"))
                .map_err(|err| invalid("DATA_DIR", &err.to_string())),
            Storage::S3(s3) => Self::s3(s3),
        }
    }

    /// Files under `dir`, created when the first one is written.
    ///
    /// # Errors
    ///
    /// [`StoreError::BadKey`] when `dir` cannot be made an absolute path.
    pub fn local(dir: &Path) -> Result<Self, StoreError> {
        let bad = || StoreError::BadKey(dir.display().to_string().into());
        let absolute = std::path::absolute(dir).map_err(|_| bad())?;
        let root = ObjectPath::from_absolute_path(absolute).map_err(|_| bad())?;
        Ok(Self(Arc::new(PrefixStore::new(
            LocalFileSystem::new(),
            root,
        ))))
    }

    /// Objects in process memory, gone with the process. For tests.
    #[must_use]
    pub fn memory() -> Self {
        Self(Arc::new(InMemory::new()))
    }

    /// Objects in the bucket of `s3`, under its prefix.
    ///
    /// # Errors
    ///
    /// A [`ConfigError`] when the settings cannot make a client.
    pub fn s3(s3: &S3) -> Result<Self, ConfigError> {
        let http = reqwest::Client::builder()
            .tls_backend_preconfigured(tls_config())
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            .build()
            .map_err(|err| {
                invalid(
                    "S3_ENDPOINT",
                    &format!("cannot build an HTTP client: {err}"),
                )
            })?;
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(&*s3.bucket)
            .with_region(&*s3.region)
            .with_access_key_id(&*s3.credentials.access_key_id)
            .with_secret_access_key(&*s3.credentials.secret_access_key)
            .with_virtual_hosted_style_request(!s3.path_style)
            .with_retry(RetryConfig {
                max_retries: MAX_RETRIES,
                retry_timeout: RETRY_WINDOW,
                ..RetryConfig::default()
            })
            .with_client_options(ClientOptions::new())
            .with_http_connector(Connector(http));
        if let Some(endpoint) = &s3.endpoint {
            builder = builder
                .with_endpoint(&**endpoint)
                .with_allow_http(endpoint.starts_with("http://"));
        }
        let store = builder
            .build()
            .map_err(|err| invalid("S3_BUCKET", &err.to_string()))?;
        Ok(match &s3.prefix {
            Some(prefix) => Self(Arc::new(PrefixStore::new(
                store,
                ObjectPath::from(&**prefix),
            ))),
            None => Self(Arc::new(store)),
        })
    }

    /// Writes `bytes` under `key`, replacing any object there.
    ///
    /// # Errors
    ///
    /// [`StoreError::BadKey`] or the backend's failure.
    pub async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<(), StoreError> {
        let path = parse(key)?;
        self.0
            .put(&path, PutPayload::from(bytes))
            .await
            .map_err(|err| StoreError::backend(key, err))?;
        Ok(())
    }

    /// Writes the local file `file` under `key` without holding it in memory: a large file goes
    /// up as a multipart upload.
    ///
    /// # Errors
    ///
    /// [`StoreError::BadKey`], reading `file`, or the backend's failure. A failed upload leaves
    /// no object behind.
    pub async fn put_file(&self, key: &str, file: &Path) -> Result<(), StoreError> {
        let mut reader = tokio::fs::File::open(file).await?;
        let mut writer = BufWriter::new(Arc::clone(&self.0), parse(key)?);
        if let Err(err) = tokio::io::copy(&mut reader, &mut writer).await {
            abort(&mut writer, key).await;
            return Err(err.into());
        }
        if let Err(err) = writer.shutdown().await {
            abort(&mut writer, key).await;
            return Err(err.into());
        }
        Ok(())
    }

    /// The size, modification time and entity tag of `key`.
    ///
    /// # Errors
    ///
    /// [`StoreError::NotFound`], [`StoreError::BadKey`] or the backend's failure.
    pub async fn head(&self, key: &str) -> Result<ObjectMeta, StoreError> {
        let meta = self
            .0
            .head(&parse(key)?)
            .await
            .map_err(|err| StoreError::backend(key, err))?;
        Ok(object_meta(meta))
    }

    /// Opens `key`, all of it or the byte `range` (end exclusive, inside the object).
    ///
    /// # Errors
    ///
    /// [`StoreError::NotFound`], [`StoreError::BadKey`] or the backend's failure.
    pub async fn get(&self, key: &str, range: Option<Range<u64>>) -> Result<Object, StoreError> {
        let options = GetOptions {
            range: range.map(GetRange::Bounded),
            ..GetOptions::default()
        };
        let result = self
            .0
            .get_opts(&parse(key)?, options)
            .await
            .map_err(|err| StoreError::backend(key, err))?;
        Ok(Object {
            meta: object_meta(result.meta.clone()),
            range: result.range.clone(),
            body: result
                .into_stream()
                .map(|chunk| chunk.map_err(io::Error::other))
                .boxed(),
        })
    }

    /// The whole content of `key` in memory. For tests and small objects.
    ///
    /// # Errors
    ///
    /// [`StoreError::NotFound`], [`StoreError::BadKey`] or the backend's failure.
    pub async fn read(&self, key: &str) -> Result<Bytes, StoreError> {
        let result = self
            .0
            .get(&parse(key)?)
            .await
            .map_err(|err| StoreError::backend(key, err))?;
        result
            .bytes()
            .await
            .map_err(|err| StoreError::backend(key, err))
    }

    /// Removes `key`. `false` when the backend reported it missing; an S3 bucket reports success
    /// for a key that was never there.
    ///
    /// # Errors
    ///
    /// [`StoreError::BadKey`] or the backend's failure.
    pub async fn delete(&self, key: &str) -> Result<bool, StoreError> {
        match self.0.delete(&parse(key)?).await {
            Ok(()) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(err) => Err(StoreError::backend(key, err)),
        }
    }

    /// Every key, sorted. A full listing: for tests and maintenance, not request paths.
    ///
    /// # Errors
    ///
    /// The backend's failure.
    pub async fn list(&self) -> Result<Vec<String>, StoreError> {
        let mut keys = Vec::new();
        let mut entries = self.0.list(None);
        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(|err| StoreError::backend("", err))?;
            keys.push(entry.location.to_string());
        }
        keys.sort();
        Ok(keys)
    }

    /// Deletes `key` when the guard drops, unless [`ObjectGuard::keep`] was called: a request
    /// that stored a file and then failed leaves nothing behind.
    pub(crate) fn guard(&self, key: &str) -> ObjectGuard {
        ObjectGuard {
            store: self.clone(),
            key: Some(key.into()),
        }
    }

    /// Writes and deletes a marker object: the bucket exists, the keys may write and delete.
    async fn probe(&self) -> Result<(), StoreError> {
        self.put(PROBE_KEY, b"tuita".to_vec()).await?;
        self.delete(PROBE_KEY).await?;
        Ok(())
    }
}

/// An object deleted on drop unless [`ObjectGuard::keep`] was called.
#[derive(Debug)]
pub(crate) struct ObjectGuard {
    store: MediaStore,
    key: Option<Box<str>>,
}

impl ObjectGuard {
    /// The object is stored for good.
    pub(crate) fn keep(&mut self) {
        self.key = None;
    }
}

impl Drop for ObjectGuard {
    fn drop(&mut self) {
        let Some(key) = self.key.take() else {
            return;
        };
        // No runtime means the process is going down; the object stays until an operator sweeps.
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let store = self.store.clone();
        runtime.spawn(async move {
            if let Err(err) = store.delete(&key).await {
                tracing::warn!(error = %err, key = &*key, "cannot remove an unused upload");
            }
        });
    }
}

fn parse(key: &str) -> Result<ObjectPath, StoreError> {
    ObjectPath::parse(key).map_err(|_| StoreError::BadKey(key.into()))
}

fn object_meta(meta: object_store::ObjectMeta) -> ObjectMeta {
    ObjectMeta {
        size: meta.size,
        modified: SystemTime::from(meta.last_modified),
        etag: meta.e_tag.map(|tag| {
            if tag.starts_with('"') || tag.starts_with("W/") {
                tag.into()
            } else {
                format!("\"{tag}\"").into()
            }
        }),
    }
}

async fn abort(writer: &mut BufWriter, key: &str) {
    if let Err(err) = writer.abort().await {
        tracing::warn!(error = %err, key, "cannot abort an upload");
    }
}

/// The startup check's failure as the variable to fix.
fn probe_failure(s3: &S3, err: &StoreError) -> ConfigError {
    let bucket = &*s3.bucket;
    match err {
        StoreError::NotFound(_) => invalid(
            "S3_BUCKET",
            &format!("bucket {bucket:?} does not exist at the endpoint"),
        ),
        StoreError::Backend(cause)
            if matches!(
                **cause,
                object_store::Error::PermissionDenied { .. }
                    | object_store::Error::Unauthenticated { .. }
            ) =>
        {
            invalid(
                "S3_ACCESS_KEY_ID",
                &format!(
                    "the access key is refused, or may not write to bucket {bucket:?} ({cause})"
                ),
            )
        }
        other => {
            let var = if s3.endpoint.is_some() {
                "S3_ENDPOINT"
            } else {
                "S3_BUCKET"
            };
            let cause = std::error::Error::source(other)
                .map_or_else(|| other.to_string(), ToString::to_string);
            invalid(var, &format!("cannot reach bucket {bucket:?} ({cause})"))
        }
    }
}

/// Hands the store our own HTTP client: rustls with the ring provider and the bundled roots, so
/// the server still links neither aws-lc nor OpenSSL.
#[derive(Debug)]
struct Connector(reqwest::Client);

impl HttpConnector for Connector {
    fn connect(&self, _options: &ClientOptions) -> object_store::Result<HttpClient> {
        Ok(HttpClient::new(self.0.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    async fn exercise(store: &MediaStore) {
        store
            .put("a.bin", b"0123456789".to_vec())
            .await
            .expect("put");
        store
            .put("sub/b.bin", b"xy".to_vec())
            .await
            .expect("put nested");
        assert_eq!(store.list().await.expect("list"), ["a.bin", "sub/b.bin"]);

        let meta = store.head("a.bin").await.expect("head");
        assert_eq!(meta.size, 10);

        let whole = store.get("a.bin", None).await.expect("get");
        assert_eq!(whole.range, 0..10);
        assert_eq!(store.read("a.bin").await.expect("read"), "0123456789");

        let part = store.get("a.bin", Some(2..5)).await.expect("range");
        assert_eq!(part.range, 2..5);
        let chunks: Vec<_> = part.body.collect().await;
        let bytes: Vec<u8> = chunks
            .into_iter()
            .flat_map(|chunk| chunk.expect("chunk"))
            .collect();
        assert_eq!(bytes, b"234");

        assert!(matches!(
            store.head("missing.bin").await,
            Err(StoreError::NotFound(_))
        ));
        assert!(matches!(
            store.put("../escape", Vec::new()).await,
            Err(StoreError::BadKey(_))
        ));
        assert!(store.delete("a.bin").await.is_ok());
        assert_eq!(store.list().await.expect("list"), ["sub/b.bin"]);
    }

    #[tokio::test]
    async fn memory_store_round_trips() {
        exercise(&MediaStore::memory()).await;
    }

    #[tokio::test]
    async fn local_store_round_trips_under_a_directory_with_a_space() {
        let dir = std::env::temp_dir().join(format!("tuita store {}", uuid::Uuid::now_v7()));
        let store = MediaStore::local(&dir.join("media")).expect("store");
        exercise(&store).await;
        assert!(
            dir.join("media/sub/b.bin").is_file(),
            "files sit at DATA_DIR/media/{{key}}"
        );
        assert!(!dir.join("media/a.bin").exists());
        assert!(!store.delete("a.bin").await.expect("again"), "already gone");
        std::fs::remove_dir_all(dir).expect("cleanup");
    }

    #[tokio::test]
    async fn put_file_uploads_a_multipart_sized_file_and_a_missing_file_fails() {
        let id = uuid::Uuid::now_v7();
        let root = std::env::temp_dir().join(format!("tuita-put-file-{id}"));
        std::fs::create_dir_all(&root).expect("temp dir");
        let file = root.join("source.bin");
        let size = 25 * 1024 * 1024;
        let content: Vec<u8> = (0..size).map(|n| (n % 251) as u8).collect();
        std::fs::write(&file, &content).expect("temp file");
        let stores = [
            MediaStore::memory(),
            MediaStore::local(&root.join("store")).expect("local store"),
        ];
        for store in stores {
            store.put_file("big.bin", &file).await.expect("upload");
            assert_eq!(store.head("big.bin").await.expect("head").size, size as u64);
            assert!(store.read("big.bin").await.expect("read") == content.as_slice());
            assert!(matches!(
                store.put_file("gone.bin", &root.join("missing")).await,
                Err(StoreError::Io(_))
            ));
            assert_eq!(store.list().await.expect("list"), ["big.bin"]);
        }
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[tokio::test]
    async fn a_dropped_guard_deletes_its_object_unless_kept() {
        let store = MediaStore::memory();
        store.put("lost.bin", vec![1]).await.expect("put");
        store.put("kept.bin", vec![1]).await.expect("put");
        drop(store.guard("lost.bin"));
        store.guard("kept.bin").keep();
        for _ in 0..100 {
            if store.list().await.expect("list") == ["kept.bin"] {
                return;
            }
            tokio::task::yield_now().await;
        }
        panic!("the guard did not delete its object");
    }

    #[test]
    fn entity_tags_are_quoted_for_http() {
        let meta = |tag: &str| object_store::ObjectMeta {
            location: ObjectPath::from("a"),
            last_modified: SystemTime::UNIX_EPOCH.into(),
            size: 1,
            e_tag: Some(tag.to_owned()),
            version: None,
        };
        assert_eq!(object_meta(meta("abc")).etag.as_deref(), Some("\"abc\""));
        assert_eq!(
            object_meta(meta("\"abc\"")).etag.as_deref(),
            Some("\"abc\"")
        );
    }

    fn s3_settings(endpoint: Option<&str>) -> S3 {
        S3 {
            bucket: "tuita-media".into(),
            region: "us-east-1".into(),
            endpoint: endpoint.map(Into::into),
            credentials: crate::storage::S3Credentials {
                access_key_id: "AKIDEXAMPLE".into(),
                secret_access_key: "s3cr3t-value".into(),
            },
            path_style: true,
            prefix: Some("media/prod".into()),
        }
    }

    #[test]
    fn an_s3_store_builds_offline_and_debug_hides_the_keys() {
        let store = MediaStore::s3(&s3_settings(Some("http://127.0.0.1:9")))
            .expect("builds without contacting the endpoint");
        let shown = format!("{store:?}");
        assert!(
            !shown.contains("s3cr3t") && !shown.contains("AKIDEXAMPLE"),
            "{shown}"
        );
    }

    #[test]
    fn a_failed_startup_check_names_the_variable_to_fix() {
        let source = || "x".into();
        let denied = StoreError::Backend(Box::new(object_store::Error::PermissionDenied {
            path: "p".to_owned(),
            source: source(),
        }));
        let unreachable = StoreError::Backend(Box::new(object_store::Error::Generic {
            store: "S3",
            source: source(),
        }));
        let with_endpoint = s3_settings(Some("http://minio:9000"));
        let aws = s3_settings(None);
        let var = |s3: &S3, err: &StoreError| probe_failure(s3, err).var;
        assert_eq!(
            var(&with_endpoint, &StoreError::NotFound("k".into())),
            "S3_BUCKET"
        );
        assert_eq!(var(&with_endpoint, &denied), "S3_ACCESS_KEY_ID");
        assert_eq!(var(&with_endpoint, &unreachable), "S3_ENDPOINT");
        assert_eq!(var(&aws, &unreachable), "S3_BUCKET");
        let shown = probe_failure(&with_endpoint, &denied).to_string();
        assert!(
            shown.starts_with("invalid S3_ACCESS_KEY_ID:") && !shown.contains("s3cr3t"),
            "{shown}"
        );
    }
}
