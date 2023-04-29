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
    fn get_by_cid<T: Decode<DagCborCodec>>(&self, cid: &Cid) -> Result<Option<T>>;
}

/// Translates writes between the MST k/v store and a backing k/v store.
// Like jmt::storage::TreeWriter
pub trait StorageWrite {
    fn put<T: Encode<DagCborCodec>>(&mut self, value: &T) -> Result<Cid>;
    // TODO: batched writes by default ?
}

/// Allows using a BTreeMap as an in-memory backing store
impl StorageRead for BTreeMap<Vec<u8>, Vec<u8>> {
    fn get_by_cid<T: Decode<DagCborCodec>>(&self, cid: &Cid) -> Result<Option<T>> {
        let Some(bytes) = self.get(&cid.to_bytes()) else {
            return Ok(None);
        };
        Ok(Some(T::decode(DagCborCodec, &mut Cursor::new(&bytes))?))
    }
}

impl StorageWrite for BTreeMap<Vec<u8>, Vec<u8>> {
    fn put<T: Encode<DagCborCodec>>(&mut self, value: &T) -> Result<Cid> {
        // Do a bit of a song and dance to cbor-encode the value to bytes
        let mut buf = Cursor::new(Vec::new());
        value.encode(DagCborCodec, &mut buf)?;
        let bytes = buf.into_inner();
        // create content id
        use libipld::cid::multihash::{Code, MultihashDigest};
        let hash = Code::Sha2_256.digest(bytes.as_ref());
        let cid = Cid::new_v1(0x71, hash);
        self.insert(cid.to_bytes(), bytes);
        Ok(cid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_round_trip() {
        let hex_simple_nd = "a2616581a4616b5820636f6d2e6578616d706c652e7265636f72642f336a716663717a6d33666f326a6170006174f66176d82a582500017112209d156bc3f3a520066252c708a9361fd3d089223842500e3713d404fdccb33cef616cf6";
        let simple_nd_bytes = hex::decode(&hex_simple_nd).unwrap();

        use libipld::{cbor::DagCborCodec, cid::multibase::Base, codec::Decode};
        use std::io::Cursor;

        let simple_nd = NodeData::decode(DagCborCodec, &mut Cursor::new(&simple_nd_bytes)).unwrap();
        let mut store = BTreeMap::new();
        let cid = store.put(&simple_nd).unwrap();

        let simple_nd_2 = store
            .get_by_cid::<NodeData>(&cid)
            .expect("btreemap is infallible")
            .expect("key is present");
        assert_eq!(simple_nd, simple_nd_2);

        assert_eq!(
            cid.to_string_of_base(Base::Base32Lower).unwrap(),
            "bafyreibj4lsc3aqnrvphp5xmrnfoorvru4wynt6lwidqbm2623a6tatzdu",
        );
    }
}
