use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Key, Nonce, Tag,
};
use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use pin_project_lite::pin_project;
use std::{
    collections::VecDeque,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use crate::backends::{
    s3::KeyMapper, AesOptions, ByteStream, Object, Options, StorageReader, StorageWriter,
};

const TAG_LEN: usize = 16;
const LEN_LEN: usize = 4;

fn try_parse_frames(
    cipher: &Aes256Gcm,
    nonce_prefix: &[u8; 8],
    counter: &mut u32,
    aad: &[u8],
    buf: &mut BytesMut,
    out_queue: &mut VecDeque<Result<Bytes, io::Error>>,
) -> Result<(), io::Error> {
    loop {
        if buf.len() < LEN_LEN {
            return Ok(());
        }

        let len = u32::from_be_bytes(buf[0..4].try_into().unwrap()) as usize;
        let needed = LEN_LEN + len + TAG_LEN;

        if buf.len() < needed {
            return Ok(());
        }

        let mut frame = buf.split_to(needed);
        let payload = &mut frame[LEN_LEN..];
        let (ct, rest) = payload.split_at_mut(len);
        let tag_bytes: &[u8] = &rest[..TAG_LEN];

        let ctr = *counter;
        *counter = counter.wrapping_add(1);

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[..8].copy_from_slice(nonce_prefix);
        nonce_bytes[8..].copy_from_slice(&ctr.to_be_bytes());

        let nonce = Nonce::from_slice(&nonce_bytes);
        let tag = Tag::from_slice(tag_bytes);

        cipher
            .decrypt_in_place_detached(nonce, aad, ct, tag)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "AES-GCM auth failed"))?;

        out_queue.push_back(Ok(Bytes::copy_from_slice(ct)));
    }
}

pin_project! {
    pub struct Aes256GcmChunkedDecrypt<S> {
        #[pin]
        input: S,
        cipher: Aes256Gcm,
        nonce_prefix: [u8; 8],
        counter: u32,
        aad: Vec<u8>,
        buf: BytesMut,
        out_queue: VecDeque<Result<Bytes, io::Error>>,
        done: bool,
    }
}

pub struct EncryptedStorage<S> {
    inner: S,
    mapper: Box<dyn KeyMapper<AesOptions>>,
}

impl<S> EncryptedStorage<S> {
    pub fn new<T: KeyMapper<AesOptions>>(inner: S, mapper: T) -> Self {
        Self {
            inner,
            mapper: Box::new(mapper),
        }
    }
}

#[async_trait::async_trait]
impl<S> StorageReader for EncryptedStorage<S>
where
    S: StorageReader,
{
    async fn get(&self, key: &str, options: &Options) -> Result<Object, io::Error> {
        let mut obj = self.inner.get(key, options).await?;
        let aes = self.mapper.get(key).await.unwrap();

        match &aes {
            Some(options) => {
                let decrypting = Aes256GcmChunkedDecrypt::new(
                    obj.stream,
                    options.key,
                    options.nonce,
                    options.counter,
                    options.aad.clone(),
                );

                obj.stream = Box::pin(decrypting);

                obj.content_length = None;

                Ok(obj)
            }
            None => Ok(obj),
        }
    }
}

impl<S> Aes256GcmChunkedDecrypt<S>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    pub fn new(
        input: S,
        key_32: [u8; 32],
        nonce_12: [u8; 12],
        counter0: u32,
        aad: Vec<u8>,
    ) -> Self {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_32));
        let mut nonce_prefix = [0u8; 8];
        nonce_prefix.copy_from_slice(&nonce_12[..8]);

        Self {
            input,
            cipher,
            nonce_prefix,
            counter: counter0,
            aad,
            buf: BytesMut::new(),
            out_queue: VecDeque::new(),
            done: false,
        }
    }
}

impl<S> Stream for Aes256GcmChunkedDecrypt<S>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    type Item = Result<Bytes, io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            if let Some(item) = this.out_queue.pop_front() {
                return Poll::Ready(Some(item));
            }

            if *this.done {
                if !this.buf.is_empty() {
                    return Poll::Ready(Some(Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "truncated encrypted stream",
                    ))));
                }
                return Poll::Ready(None);
            }

            match this.input.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    *this.done = true;
                    if let Err(e) = try_parse_frames(
                        this.cipher,
                        this.nonce_prefix,
                        this.counter,
                        this.aad,
                        this.buf,
                        this.out_queue,
                    ) {
                        return Poll::Ready(Some(Err(e)));
                    }
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(Some(Ok(bytes))) => {
                    this.buf.extend_from_slice(&bytes);
                    if let Err(e) = try_parse_frames(
                        this.cipher,
                        this.nonce_prefix,
                        this.counter,
                        this.aad,
                        this.buf,
                        this.out_queue,
                    ) {
                        return Poll::Ready(Some(Err(e)));
                    }
                }
            }
        }
    }
}

fn encrypt_one_frame(
    cipher: &Aes256Gcm,
    nonce_prefix: &[u8; 8],
    counter: &mut u32,
    aad: &[u8],
    pt: &[u8],
) -> Result<Bytes, io::Error> {
    let len = pt.len();
    if len > u32::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame too large",
        ));
    }

    let ctr = *counter;
    *counter = counter.wrapping_add(1);

    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(nonce_prefix);
    nonce_bytes[8..].copy_from_slice(&ctr.to_be_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut frame = Vec::with_capacity(LEN_LEN + len + TAG_LEN);
    frame.extend_from_slice(&(len as u32).to_be_bytes());
    frame.extend_from_slice(pt);

    let ct = &mut frame[LEN_LEN..LEN_LEN + len];
    let tag: Tag = cipher
        .encrypt_in_place_detached(nonce, aad, ct)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "AES-GCM encrypt failed"))?;

    frame.extend_from_slice(tag.as_slice());
    Ok(Bytes::from(frame))
}

pin_project! {
    pub struct Aes256GcmChunkedEncrypt<S> {
        #[pin]
        input: S,
        cipher: Aes256Gcm,
        nonce_prefix: [u8; 8],
        counter: u32,
        aad: Vec<u8>,
        out_queue: VecDeque<Result<Bytes, io::Error>>,
        done: bool,
    }
}

impl<S> Aes256GcmChunkedEncrypt<S>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    pub fn new(
        input: S,
        key_32: [u8; 32],
        nonce_12: [u8; 12],
        counter0: u32,
        aad: Vec<u8>,
    ) -> Self {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_32));
        let mut nonce_prefix = [0u8; 8];
        nonce_prefix.copy_from_slice(&nonce_12[..8]);

        Self {
            input,
            cipher,
            nonce_prefix,
            counter: counter0,
            aad,
            out_queue: VecDeque::new(),
            done: false,
        }
    }
}

impl<S> Stream for Aes256GcmChunkedEncrypt<S>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    type Item = Result<Bytes, io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if let Some(item) = this.out_queue.pop_front() {
            return Poll::Ready(Some(item));
        }

        if *this.done {
            return Poll::Ready(None);
        }

        match this.input.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                *this.done = true;
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Some(Ok(bytes))) => {
                let framed = encrypt_one_frame(
                    this.cipher,
                    this.nonce_prefix,
                    this.counter,
                    this.aad,
                    &bytes,
                );

                Poll::Ready(Some(framed))
            }
        }
    }
}

#[async_trait::async_trait]
impl<S> StorageWriter for EncryptedStorage<S>
where
    S: StorageWriter,
{
    async fn write(&self, key: &str, stream: ByteStream) -> Result<(), std::io::Error> {
        let aes_options = AesOptions::new();
        self.mapper.set(key, aes_options.clone()).await.unwrap();
        let aad = aes_options.aad.clone();
        let encrypting = Aes256GcmChunkedEncrypt::new(
            stream,
            aes_options.key,
            aes_options.nonce,
            aes_options.counter,
            aad,
        );

        let enc_stream: ByteStream = Box::pin(encrypting);
        self.inner.write(key, enc_stream).await
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), std::io::Error> {
        self.mapper.rename(orig_key, target_key).await.unwrap();
        self.inner.rename(orig_key, target_key).await
    }
}
