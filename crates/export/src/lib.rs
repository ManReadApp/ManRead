use bytes::Bytes;
use prost::{DecodeError, Message};

pub mod manga {
    include!(concat!(env!("OUT_DIR"), "/manga.rs"));
}

pub fn from_bytes<M: Message + Default>(b: &[u8]) -> M {
    try_from_bytes(b).unwrap()
}

pub fn try_from_bytes<M: Message + Default>(b: &[u8]) -> Result<M, DecodeError> {
    M::decode(b)
}

pub fn to_bytes<M: Message>(m: &M) -> Bytes {
    let mut v = Vec::with_capacity(m.encoded_len());
    m.encode(&mut v).unwrap();
    Bytes::from(v)
}
