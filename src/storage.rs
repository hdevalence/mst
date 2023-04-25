use std::{collections::BTreeMap, io};

use cid::Cid;
use serde::{Deserialize, Deserializer as _, Serialize};

// go NodeEntry
pub enum Node {
    Leaf {
        key: Vec<u8>,
        val: Vec<u8>,
    },
    Internal {
        // todo
    },
}

#[derive(Clone, Debug)]
struct NodeData {
    left: Option<Cid>,
    entries: Vec<TreeEntry>,
}

#[derive(Clone, Debug)]
struct TreeEntry {
    prefix_len: usize,
    key_suffix: Vec<u8>,
    val: Cid,
    tree: Option<Cid>,
}

impl NodeData {
    fn read_cbor<R: io::Read>(reader: R) -> io::Result<Self> {
        // todo
        todo!()
    }
}

impl TreeEntry {}

// not sure this is actually any better than manually invoking serializer/deserializer methods ... it may be much worse
mod helper {
    use super::*;

    use serde_cbor::tags::Tagged;

    #[derive(Serialize, Deserialize)]
    struct CborNodeData {
        l: Option<Tagged<serde_bytes::ByteBuf>>,
        e: Vec<CborTreeEntry>,
    }

    impl From<NodeData> for CborNodeData {
        fn from(value: NodeData) -> Self {
            Self {
                l: value.left.map(cid_to_cbor),
                e: value.entries.into_iter().map(CborTreeEntry::from).collect(),
            }
        }
    }

    impl TryFrom<CborNodeData> for NodeData {
        type Error = anyhow::Error;
        fn try_from(value: CborNodeData) -> Result<Self, Self::Error> {
            Ok(Self {
                left: value.l.map(cid_from_cbor).transpose()?,
                entries: value
                    .e
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    #[derive(Serialize, Deserialize)]
    struct CborTreeEntry {
        p: i64,
        k: serde_bytes::ByteBuf,
        v: Tagged<serde_bytes::ByteBuf>,
        t: Option<Tagged<serde_bytes::ByteBuf>>,
    }

    fn cid_to_cbor(cid: Cid) -> Tagged<serde_bytes::ByteBuf> {
        Tagged::new(
            Some(42),
            serde_bytes::ByteBuf::from(
                std::iter::once(0u8)
                    .chain(cid.to_bytes())
                    .collect::<Vec<u8>>(),
            ),
        )
    }

    fn cid_from_cbor(cid: Tagged<serde_bytes::ByteBuf>) -> anyhow::Result<Cid> {
        if let Some(42) = cid.tag {
            Ok(Cid::try_from(&cid.value[1..])?)
        } else {
            Err(anyhow::anyhow!(
                "wrong tag for cbor cid, expected 42, found {:?}",
                cid.tag
            ))
        }
    }

    impl From<TreeEntry> for CborTreeEntry {
        fn from(e: TreeEntry) -> Self {
            Self {
                p: e.prefix_len as i64,
                k: serde_bytes::ByteBuf::from(e.key_suffix),
                v: cid_to_cbor(e.val),
                t: e.tree.map(cid_to_cbor),
            }
        }
    }

    impl TryFrom<CborTreeEntry> for TreeEntry {
        type Error = anyhow::Error;
        fn try_from(value: CborTreeEntry) -> Result<Self, Self::Error> {
            Ok(Self {
                prefix_len: value.p as usize,
                key_suffix: value.k.into_vec(),
                val: cid_from_cbor(value.v)?,
                tree: value.t.map(cid_from_cbor).transpose()?,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn deserialize_test_vector() {
            let hex_simple_nd = "a2616581a4616b5820636f6d2e6578616d706c652e7265636f72642f336a716663717a6d33666f326a6170006174f66176d82a582500017112209d156bc3f3a520066252c708a9361fd3d089223842500e3713d404fdccb33cef616cf6";
            let simple_nd_bytes = hex::decode(&hex_simple_nd).unwrap();

            let tokens = minicbor::decode::Tokenizer::new(&simple_nd_bytes)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            println!("{:?}", tokens);

            let ciborium_value: ciborium::value::Value =
                ciborium::de::from_reader(&simple_nd_bytes[..]).unwrap();
            println!("ciborium {:#?}", ciborium_value);

            let simple_nd_value =
                serde_cbor::from_slice::<serde_cbor::Value>(&simple_nd_bytes).unwrap();
            println!("{:?}", simple_nd_value);
            let simple_nd_bytes_2 = serde_cbor::to_vec(&simple_nd_value).unwrap();
            let hex_simple_nd_2 = hex::encode(&simple_nd_bytes_2);
            println!("{}", hex_simple_nd_2);
            assert_eq!(simple_nd_bytes, simple_nd_bytes_2);

            let simple_nd_cbor = serde_cbor::from_slice::<CborNodeData>(&simple_nd_bytes).unwrap();
            let simple_nd = NodeData::try_from(simple_nd_cbor).unwrap();
            println!("{:?}", simple_nd);

            panic!();
        }
    }
}

impl TreeEntry {
    pub fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        // todo
        todo!()
    }
}

/// Translates reads between the MST k/v store and a backing k/v store.
// Like jmt::storage::TreeReader
pub trait StorageRead {
    // TODO: what methods do we need to read internal nodes?
}

/// Translates writes between the MST k/v store and a backing k/v store.
// Like jmt::storage::TreeWriter
pub trait StorageWrite {
    // TODO: batched writes by default ?
}

/// Allows using a BTreeMap as an in-memory backing store
impl StorageRead for BTreeMap<Vec<u8>, Vec<u8>> {}
impl StorageWrite for BTreeMap<Vec<u8>, Vec<u8>> {}
