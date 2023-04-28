use libipld::{
    cbor::DagCborCodec,
    codec::{Decode, Encode},
};
use std::io::Cursor;

use libipld::{Cid, DagCbor};
use std::collections::BTreeMap;

use anyhow::Result;

use crate::node::{NodeData, TreeEntry};

/// Translates reads between the MST k/v store and a backing k/v store.
// Like jmt::storage::TreeReader
pub trait StorageRead {
    // TODO: what methods do we need to read internal nodes?
    fn get<T: Decode<DagCborCodec>>(&self, cid: &Cid) -> Result<Option<T>>;
}

/// Translates writes between the MST k/v store and a backing k/v store.
// Like jmt::storage::TreeWriter
pub trait StorageWrite {
    fn put<T: Encode<DagCborCodec>>(&self, value: &T) -> Result<()>;
    // TODO: batched writes by default ?
}

/// Allows using a BTreeMap as an in-memory backing store
impl StorageRead for BTreeMap<Vec<u8>, Vec<u8>> {
    fn get<T: Decode<DagCborCodec>>(&self, cid: &Cid) -> Result<Option<T>> {
        let Some(bytes) = self.get(&cid.to_bytes()) else {
            return Ok(None);
        };
        Ok(Some(T::decode(DagCborCodec, &mut Cursor::new(&bytes))?))
    }
}

impl StorageWrite for BTreeMap<Vec<u8>, Vec<u8>> {
    fn put<T: Encode<DagCborCodec>>(&self, value: &T) -> Result<()> {
        // Do a bit of a song and dance to cbor-encode the value to bytes
        let mut buf = Cursor::new(Vec::new());
        value.encode(DagCborCodec, &mut buf)?;
        let bytes = buf.into_inner();
        // create content id 
        
        todo!()
    }
}
