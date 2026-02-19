use std::{
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use futures_util::{stream, TryStreamExt as _};

use crate::backends::{ByteStream, KeyValueStore, Object, Options, StorageReader, StorageWriter};

pub struct ContentLengthStorage<S, K> {
    inner: S,
    content_lengths: K,
}

impl<S, K> ContentLengthStorage<S, K> {
    pub fn new(inner: S, content_lengths: K) -> Self {
        Self {
            inner,
            content_lengths,
        }
    }
}

fn content_length_only_dummy_stream() -> ByteStream {
    Box::pin(stream::once(async {
        Err(io::Error::other(
            "content_length_only object stream should never be consumed",
        ))
    }))
}

#[async_trait::async_trait]
impl<S, K> StorageReader for ContentLengthStorage<S, K>
where
    S: StorageReader,
    K: KeyValueStore<u64, Error = io::Error>,
{
    async fn get(&self, key: &str, options: &Options) -> Result<Object, io::Error> {
        if options.content_length_only {
            let content_length = self
                .content_lengths
                .get(key)
                .await?
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not found"))?;

            return Ok(Object {
                stream: content_length_only_dummy_stream(),
                content_length: Some(content_length),
                content_type: None,
                etag: None,
                last_modified: None,
            });
        }

        let mut obj = self.inner.get(key, options).await?;

        if let Some(content_length) = self.content_lengths.get(key).await? {
            obj.content_length = Some(content_length);
        }

        Ok(obj)
    }
}

#[async_trait::async_trait]
impl<S, K> StorageWriter for ContentLengthStorage<S, K>
where
    S: StorageWriter,
    K: KeyValueStore<u64, Error = io::Error>,
{
    async fn write(&self, key: &str, stream: ByteStream) -> Result<(), io::Error> {
        let observed_len = Arc::new(AtomicU64::new(0));
        let counter = observed_len.clone();
        let measured_stream = stream.inspect_ok(move |chunk| {
            counter.fetch_add(chunk.len() as u64, Ordering::Relaxed);
        });

        self.inner.write(key, Box::pin(measured_stream)).await?;
        self.content_lengths
            .set(key, observed_len.load(Ordering::Relaxed))
            .await
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
        self.inner.rename(orig_key, target_key).await?;
        self.content_lengths.rename(orig_key, target_key).await
    }

    async fn delete(&self, key: &str) -> Result<(), io::Error> {
        self.inner.delete(key).await?;
        let _ = self.content_lengths.remove(key).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    use bytes::Bytes;
    use futures_util::{stream, StreamExt as _};
    use tokio::sync::RwLock;

    use super::*;
    use crate::backends::MemStorage;

    fn stream_from_parts(parts: Vec<Vec<u8>>) -> ByteStream {
        let chunks = parts
            .into_iter()
            .map(|part| Ok::<Bytes, io::Error>(Bytes::from(part)));
        Box::pin(stream::iter(chunks))
    }

    async fn read_all(mut stream: ByteStream) -> Result<Vec<u8>, io::Error> {
        let mut out = Vec::new();
        while let Some(chunk) = stream.next().await {
            out.extend_from_slice(&chunk?);
        }
        Ok(out)
    }

    struct TestKeyValueStore {
        map: Arc<RwLock<HashMap<String, u64>>>,
    }

    #[async_trait::async_trait]
    impl KeyValueStore<u64> for TestKeyValueStore {
        type Error = io::Error;

        async fn get(&self, key: &str) -> Result<Option<u64>, io::Error> {
            Ok(self.map.read().await.get(key).copied())
        }

        async fn set(&self, key: &str, value: u64) -> Result<(), io::Error> {
            self.map.write().await.insert(key.to_owned(), value);
            Ok(())
        }

        async fn remove(&self, key: &str) -> Result<Option<u64>, io::Error> {
            Ok(self.map.write().await.remove(key))
        }
    }

    struct MisreportingStorage {
        inner: MemStorage,
        reported_length: u64,
    }

    #[async_trait::async_trait]
    impl StorageWriter for MisreportingStorage {
        async fn write(&self, key: &str, stream: ByteStream) -> Result<(), io::Error> {
            self.inner.write(key, stream).await
        }

        async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
            self.inner.rename(orig_key, target_key).await
        }

        async fn delete(&self, key: &str) -> Result<(), io::Error> {
            self.inner.delete(key).await
        }
    }

    #[async_trait::async_trait]
    impl StorageReader for MisreportingStorage {
        async fn get(&self, key: &str, options: &Options) -> Result<Object, io::Error> {
            let mut obj = self.inner.get(key, options).await?;
            obj.content_length = Some(self.reported_length);
            Ok(obj)
        }
    }

    struct InnerReaderMustNotRun {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl StorageReader for InnerReaderMustNotRun {
        async fn get(&self, _key: &str, _options: &Options) -> Result<Object, io::Error> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(io::Error::other(
                "inner get should not run for content_length_only",
            ))
        }
    }

    #[tokio::test]
    async fn write_stores_content_length_and_get_overrides_inner_length() -> Result<(), io::Error> {
        let payload = b"content-length-write-tracking";
        let map = Arc::new(RwLock::new(HashMap::new()));
        let kv = TestKeyValueStore { map: map.clone() };
        let storage = ContentLengthStorage::new(
            MisreportingStorage {
                inner: MemStorage::new(),
                reported_length: 1,
            },
            kv,
        );

        storage
            .write(
                "items/a",
                stream_from_parts(vec![
                    payload[..8].to_vec(),
                    payload[8..17].to_vec(),
                    payload[17..].to_vec(),
                ]),
            )
            .await?;

        assert_eq!(
            map.read().await.get("items/a").copied(),
            Some(payload.len() as u64)
        );

        let obj = storage.get("items/a", &Options::default()).await?;
        assert_eq!(obj.content_length, Some(payload.len() as u64));
        assert_eq!(read_all(obj.stream).await?, payload);
        Ok(())
    }

    #[tokio::test]
    async fn rename_moves_stored_content_length() -> Result<(), io::Error> {
        let payload = b"rename-content-length";
        let map = Arc::new(RwLock::new(HashMap::new()));
        let kv = TestKeyValueStore { map: map.clone() };
        let storage = ContentLengthStorage::new(MemStorage::new(), kv);

        storage
            .write(
                "items/tmp",
                stream_from_parts(vec![payload[..6].to_vec(), payload[6..].to_vec()]),
            )
            .await?;
        storage.rename("items/tmp", "items/final").await?;

        let lengths = map.read().await;
        assert_eq!(lengths.get("items/tmp"), None);
        assert_eq!(
            lengths.get("items/final").copied(),
            Some(payload.len() as u64)
        );
        drop(lengths);

        let obj = storage.get("items/final", &Options::default()).await?;
        assert_eq!(obj.content_length, Some(payload.len() as u64));
        assert_eq!(read_all(obj.stream).await?, payload);
        Ok(())
    }

    #[tokio::test]
    async fn delete_removes_stored_content_length() -> Result<(), io::Error> {
        let payload = b"delete-content-length";
        let map = Arc::new(RwLock::new(HashMap::new()));
        let kv = TestKeyValueStore { map: map.clone() };
        let storage = ContentLengthStorage::new(MemStorage::new(), kv);

        storage
            .write(
                "items/to-delete",
                stream_from_parts(vec![payload[..7].to_vec(), payload[7..].to_vec()]),
            )
            .await?;
        storage.delete("items/to-delete").await?;

        assert_eq!(map.read().await.get("items/to-delete"), None);
        Ok(())
    }

    #[tokio::test]
    async fn content_length_only_returns_stored_length_without_inner_read() -> Result<(), io::Error>
    {
        let map = Arc::new(RwLock::new(HashMap::new()));
        map.write().await.insert("items/a".to_owned(), 42);
        let kv = TestKeyValueStore { map };
        let calls = Arc::new(AtomicUsize::new(0));
        let storage = ContentLengthStorage::new(
            InnerReaderMustNotRun {
                calls: calls.clone(),
            },
            kv,
        );

        let obj = storage
            .get(
                "items/a",
                &Options {
                    content_length_only: true,
                    ..Options::default()
                },
            )
            .await?;
        assert_eq!(obj.content_length, Some(42));
        assert_eq!(calls.load(Ordering::Relaxed), 0);

        let mut stream = obj.stream;
        let err = stream
            .next()
            .await
            .expect("dummy stream should produce one value")
            .expect_err("dummy stream value should be an error");
        assert_eq!(err.kind(), io::ErrorKind::Other);
        Ok(())
    }

    #[tokio::test]
    async fn content_length_only_missing_length_returns_not_found_without_inner_read() {
        let map = Arc::new(RwLock::new(HashMap::new()));
        let kv = TestKeyValueStore { map };
        let calls = Arc::new(AtomicUsize::new(0));
        let storage = ContentLengthStorage::new(
            InnerReaderMustNotRun {
                calls: calls.clone(),
            },
            kv,
        );

        let err = match storage
            .get(
                "items/missing",
                &Options {
                    content_length_only: true,
                    ..Options::default()
                },
            )
            .await
        {
            Ok(_) => panic!("missing content length should return not found"),
            Err(err) => err,
        };
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }
}
