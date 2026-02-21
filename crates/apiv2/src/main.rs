use std::{io, sync::Arc};

use db::DbHandle;
use storage::{StorageReader, StorageWriter};

use crate::init::env::{Config, S3UploadAclConfig};

mod actions;
pub mod error;
mod init;
mod routes;

trait StorageBackend: StorageReader + StorageWriter {}
impl<T> StorageBackend for T where T: StorageReader + StorageWriter {}

fn map_upload_acl(acl: S3UploadAclConfig) -> storage::S3UploadAcl {
    match acl {
        S3UploadAclConfig::InheritBucket => storage::S3UploadAcl::InheritBucket,
        S3UploadAclConfig::Private => storage::S3UploadAcl::Private,
        S3UploadAclConfig::PublicRead => storage::S3UploadAcl::PublicRead,
    }
}

async fn build_storage_backend(
    config: &Config,
    handle: &DbHandle,
) -> io::Result<Arc<dyn StorageBackend + Send + Sync>> {
    let mut backend: Arc<dyn StorageBackend + Send + Sync> = if config.storage.local {
        Arc::new(storage::DiskStorage::new(&config.root_folder))
    } else {
        let s3_cfg = &config.storage.s3;
        let mut options =
            storage::S3StorageOptions::new(s3_cfg.bucket.clone(), s3_cfg.region.clone());
        options.endpoint = s3_cfg.endpoint.clone();
        options.access_key_id = s3_cfg.access_key_id.clone();
        options.secret_access_key = s3_cfg.secret_access_key.clone();
        options.session_token = s3_cfg.session_token.clone();
        options.force_path_style = s3_cfg.force_path_style;
        options.upload_acl = map_upload_acl(s3_cfg.upload_acl.clone());
        Arc::new(storage::S3Storage::new_with_key_map(options, handle.kv("s3")).await?)
    };

    if config.storage.encryption {
        backend = Arc::new(storage::EncryptedStorage::new(
            backend,
            handle.kv("aes_gcm"),
        ));
    }

    if !config.storage.local {
        backend = Arc::new(storage::CacheBackend::new(&config.root_folder, backend));
    }

    backend = Arc::new(storage::ContentLengthStorage::new(
        backend,
        handle.kv("content_length"),
    ));

    Ok(backend)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Arc::new(init::env::get_env()?);
    init::logger::init_logger(&config.rust_log)
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    let dbs = db::init_db(Default::default())
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    let backend = build_storage_backend(config.as_ref(), &dbs).await?;
    let reader: Arc<dyn StorageReader + Send + Sync> = backend.clone();
    let writer: Arc<dyn StorageWriter + Send + Sync> = backend;
    let storage = storage::StorageSystem::new_with_rw(&config.root_folder, reader, writer, 5)
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    init::server::init_server(
        config.port,
        config.https_port,
        config,
        Arc::new(storage),
        dbs,
    )?
    .await
}
