use std::{
    io,
    path::{Component, Path},
};

use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream as S3ByteStream,
    types::ObjectCannedAcl,
    Client,
};
use bytes::BytesMut;
use futures_util::TryStreamExt as _;
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::backends::{ByteStream, KeyValueStore, Object, Options, StorageReader, StorageWriter};

#[derive(Clone, Copy, Debug, Default)]
pub enum S3UploadAcl {
    #[default]
    InheritBucket,
    Private,
    PublicRead,
}

#[derive(Clone, Debug)]
pub struct S3StorageOptions {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub force_path_style: bool,
    pub upload_acl: S3UploadAcl,
}

impl S3StorageOptions {
    pub fn new(bucket: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            region: region.into(),
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            force_path_style: false,
            upload_acl: S3UploadAcl::InheritBucket,
        }
    }
}

pub struct S3Storage<K> {
    client: Client,
    bucket: String,
    upload_acl: S3UploadAcl,
    key_map: K,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Payload {
    val: String,
}

impl Payload {
    fn new(val: String) -> Self {
        Self { val }
    }
}

impl<K> S3Storage<K>
where
    K: KeyValueStore<Payload>,
{
    pub async fn new_with_key_map(
        options: S3StorageOptions,
        key_map: K,
    ) -> Result<Self, io::Error> {
        if options.bucket.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "s3 bucket cannot be empty",
            ));
        }
        if options.region.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "s3 region cannot be empty",
            ));
        }

        let has_access_key = options.access_key_id.is_some();
        let has_secret_key = options.secret_access_key.is_some();
        if has_access_key ^ has_secret_key {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "both access_key_id and secret_access_key must be set together",
            ));
        }

        let mut shared = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(options.region.clone()));

        if let (Some(access_key), Some(secret_key)) = (
            options.access_key_id.clone(),
            options.secret_access_key.clone(),
        ) {
            shared = shared.credentials_provider(Credentials::new(
                access_key,
                secret_key,
                options.session_token.clone(),
                None,
                "storage-s3-config",
            ));
        }

        let shared = shared.load().await;

        let mut config =
            aws_sdk_s3::config::Builder::from(&shared).force_path_style(options.force_path_style);
        if let Some(endpoint) = options.endpoint.as_deref() {
            config = config.endpoint_url(endpoint);
        }

        let client = Client::from_conf(config.build());
        Ok(Self {
            client,
            bucket: options.bucket,
            upload_acl: options.upload_acl,
            key_map,
        })
    }

    fn validate_key(key: &str) -> Result<(), io::Error> {
        let rel = Path::new(key);
        if rel.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "storage key cannot be empty",
            ));
        }

        for comp in rel.components() {
            match comp {
                Component::Normal(_) => {}
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "storage key must be a safe relative path",
                    ));
                }
            }
        }

        Ok(())
    }

    fn canned_acl(&self) -> Option<ObjectCannedAcl> {
        match self.upload_acl {
            S3UploadAcl::InheritBucket => None,
            S3UploadAcl::Private => Some(ObjectCannedAcl::Private),
            S3UploadAcl::PublicRead => Some(ObjectCannedAcl::PublicRead),
        }
    }

    fn s3_err(context: &str, err: impl std::fmt::Display) -> io::Error {
        let msg = err.to_string();
        let kind = if msg.contains("NoSuchKey") || msg.contains("NotFound") {
            io::ErrorKind::NotFound
        } else {
            io::ErrorKind::Other
        };
        io::Error::new(kind, format!("s3 {context} failed: {msg}"))
    }

    fn kv_err(context: &str, err: impl std::fmt::Display) -> io::Error {
        io::Error::other(format!("s3 key mapping {context} failed: {err}"))
    }

    fn generate_object_id() -> String {
        format!("objects/{}", Uuid::new_v4())
    }

    async fn mapped_object_id(map: &K, key: &str) -> Result<Option<String>, io::Error> {
        map.get(key)
            .await
            .map_err(|err| Self::kv_err("get", err))
            .map(|v| v.map(|v| v.val))
    }

    async fn required_object_id(map: &K, key: &str) -> Result<String, io::Error> {
        Self::mapped_object_id(map, key).await?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("storage key not found: {key}"),
            )
        })
    }

    async fn rename_key_mapping_only(
        map: &K,
        orig_key: &str,
        target_key: &str,
    ) -> Result<Option<String>, io::Error> {
        if orig_key == target_key {
            return Ok(None);
        }

        let source_object_id = Self::required_object_id(map, orig_key).await?;
        let replaced_target_object_id = Self::mapped_object_id(map, target_key).await?;

        map.rename(orig_key, target_key)
            .await
            .map_err(|err| Self::kv_err("rename", err))?;

        Ok(replaced_target_object_id.filter(|existing| existing != &source_object_id))
    }

    async fn delete_object_by_id(&self, object_id: &str) -> Result<(), io::Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object_id)
            .send()
            .await
            .map_err(|err| Self::s3_err("delete_object", err))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<K> StorageWriter for S3Storage<K>
where
    K: KeyValueStore<Payload>,
{
    async fn write(&self, key: &str, mut stream: ByteStream) -> Result<(), io::Error> {
        Self::validate_key(key)?;

        let mut data = BytesMut::new();
        while let Some(chunk) = stream.try_next().await? {
            data.extend_from_slice(&chunk);
        }

        let object_id = Self::generate_object_id();
        let mut req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&object_id)
            .body(S3ByteStream::from(data.freeze().to_vec()));

        if let Some(content_type) = mime_guess::from_path(key).first_raw() {
            req = req.content_type(content_type);
        }
        if let Some(acl) = self.canned_acl() {
            req = req.acl(acl);
        }

        req.send()
            .await
            .map_err(|err| Self::s3_err("put_object", err))?;

        let previous_object_id = Self::mapped_object_id(&self.key_map, key).await?;
        self.key_map
            .set(key, Payload::new(object_id.clone()))
            .await
            .map_err(|err| Self::kv_err("set", err))?;

        if let Some(previous_object_id) = previous_object_id {
            if previous_object_id != object_id {
                let delete_result = self.delete_object_by_id(&previous_object_id).await;
                if let Err(err) = delete_result {
                    if err.kind() != io::ErrorKind::NotFound {
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
        Self::validate_key(orig_key)?;
        Self::validate_key(target_key)?;

        let replaced_target_object_id =
            Self::rename_key_mapping_only(&self.key_map, orig_key, target_key).await?;

        if let Some(object_id) = replaced_target_object_id {
            let delete_result = self.delete_object_by_id(&object_id).await;
            if let Err(err) = delete_result {
                if err.kind() != io::ErrorKind::NotFound {
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), io::Error> {
        Self::validate_key(key)?;

        let object_id = self
            .key_map
            .remove(key)
            .await
            .map_err(|err| Self::kv_err("remove", err))?;

        if let Some(object_id) = object_id {
            let delete_result = self.delete_object_by_id(&object_id.val).await;
            if let Err(err) = delete_result {
                if err.kind() != io::ErrorKind::NotFound {
                    return Err(err);
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<K> StorageReader for S3Storage<K>
where
    K: KeyValueStore<Payload>,
{
    async fn get(&self, key: &str, _: &Options) -> Result<Object, io::Error> {
        Self::validate_key(key)?;
        let object_id = Self::required_object_id(&self.key_map, key).await?;

        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_id)
            .send()
            .await
            .map_err(|err| Self::s3_err("get_object", err))?;

        let content_length = out
            .content_length()
            .and_then(|value| u64::try_from(value).ok());
        let content_type = out
            .content_type()
            .and_then(|value| value.parse::<mime::Mime>().ok());
        let etag = out.e_tag().map(ToString::to_string);
        let last_modified = out
            .last_modified()
            .and_then(|value| std::time::SystemTime::try_from(*value).ok());
        let stream: ByteStream = Box::pin(ReaderStream::new(out.body.into_async_read()));

        Ok(Object {
            stream,
            content_length,
            content_type,
            etag,
            last_modified,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::S3Storage;
    use crate::backends::{s3::Payload, InMemoryKeyValueStore, KeyValueStore};

    type DefaultS3Storage = S3Storage<InMemoryKeyValueStore<Payload>>;

    #[test]
    fn validate_key_accepts_safe_relative_paths() {
        assert!(DefaultS3Storage::validate_key("temp/abc").is_ok());
        assert!(DefaultS3Storage::validate_key("mangas/a/b/c.png").is_ok());
    }

    #[test]
    fn validate_key_rejects_unsafe_paths() {
        assert!(DefaultS3Storage::validate_key("").is_err());
        assert!(DefaultS3Storage::validate_key("../escape").is_err());
        assert!(DefaultS3Storage::validate_key("/absolute").is_err());
    }

    #[tokio::test]
    async fn rename_key_mapping_moves_id_only() {
        let map = InMemoryKeyValueStore::new();
        map.set("tmp/key", Payload::new("objects/id-1".to_string()))
            .await
            .unwrap();

        let replaced = DefaultS3Storage::rename_key_mapping_only(&map, "tmp/key", "final/key")
            .await
            .unwrap();

        assert_eq!(replaced, None);
        assert_eq!(map.get("tmp/key").await.unwrap(), None);
        assert_eq!(
            map.get("final/key").await.unwrap(),
            Some("objects/id-1".to_string())
        );
    }

    #[tokio::test]
    async fn rename_key_mapping_returns_replaced_target_id() {
        let map = InMemoryKeyValueStore::new();
        map.set("tmp/key", Payload::new("objects/id-1".to_string()))
            .await
            .unwrap();
        map.set("final/key", Payload::new("objects/id-2".to_string()))
            .await
            .unwrap();

        let replaced = DefaultS3Storage::rename_key_mapping_only(&map, "tmp/key", "final/key")
            .await
            .unwrap();

        assert_eq!(replaced, Some("objects/id-2".to_string()));
        assert_eq!(map.get("tmp/key").await.unwrap(), None);
        assert_eq!(
            map.get("final/key").await.unwrap(),
            Some("objects/id-1".to_string())
        );
    }

    #[tokio::test]
    async fn rename_key_mapping_requires_existing_source() {
        let map = InMemoryKeyValueStore::<Payload>::new();

        let err = DefaultS3Storage::rename_key_mapping_only(&map, "tmp/key", "final/key")
            .await
            .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
