use std::{io, time::Duration};

use tokio::time::sleep;

use crate::backends::{GenerateOptions, Object, Options, StorageReader, StorageWriter};

pub struct DelayStorage<S> {
    inner: S,
    read_delay: Duration,
    write_delay: Duration,
    rename_delay: Duration,
}

impl<S> DelayStorage<S> {
    pub fn new(inner: S, delay: Duration) -> Self {
        Self {
            inner,
            read_delay: delay,
            write_delay: delay,
            rename_delay: delay,
        }
    }

    pub fn with_delays(
        inner: S,
        read_delay: Duration,
        write_delay: Duration,
        rename_delay: Duration,
    ) -> Self {
        Self {
            inner,
            read_delay,
            write_delay,
            rename_delay,
        }
    }
}

impl<S: GenerateOptions> GenerateOptions for DelayStorage<S> {
    fn generate_options(&self) -> Options {
        self.inner.generate_options()
    }
}

#[async_trait::async_trait]
impl<S> StorageReader for DelayStorage<S>
where
    S: StorageReader,
{
    async fn get(&self, key: &str, options: &Options) -> Result<Object, io::Error> {
        sleep(self.read_delay).await;
        self.inner.get(key, options).await
    }
}

#[async_trait::async_trait]
impl<S> StorageWriter for DelayStorage<S>
where
    S: StorageWriter,
{
    async fn write(
        &self,
        key: &str,
        options: &Options,
        stream: crate::backends::ByteStream,
    ) -> Result<(), io::Error> {
        sleep(self.write_delay).await;
        self.inner.write(key, options, stream).await
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
        sleep(self.rename_delay).await;
        self.inner.rename(orig_key, target_key).await
    }
}
