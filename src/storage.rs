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

impl TreeEntry {
    pub fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        // todo
        todo!()
    }
}

mod helper2 {
    use super::*;

    use ciborium::value::Value;

    fn cid_to_cbor(cid: Cid) -> Value {
        Value::Tag(
            42,
            Box::new(Value::Bytes(
                std::iter::once(0u8)
                    .chain(cid.to_bytes())
                    .collect::<Vec<u8>>(),
            )),
        )
    }

    fn cid_from_cbor(cid: Value) -> anyhow::Result<Cid> {
        match cid {
            Value::Tag(42, cid) => match *cid {
                Value::Bytes(cid) => Ok(Cid::try_from(&cid[1..])?),
                something_else => Err(anyhow::anyhow!(
                    "expected Value::Bytes for cbor cid, found {:?}",
                    something_else
                )),
            },
            _ => Err(anyhow::anyhow!(
                "wrong tag for cbor cid, expected 42, found {:?}",
                cid
            )),
        }
    }

    impl From<NodeData> for Value {
        fn from(value: NodeData) -> Self {
            Value::Map(vec![
                (
                    Value::Text("e".to_owned()),
                    Value::Array(value.entries.into_iter().map(Value::from).collect()),
                ),
                (
                    Value::Text("l".to_owned()),
                    value.left.map(cid_to_cbor).unwrap_or(Value::Null),
                ),
            ])
        }
    }

    impl TryFrom<Value> for NodeData {
        type Error = anyhow::Error;

        fn try_from(value: Value) -> Result<Self, Self::Error> {
            let Value::Map(map) = value else {
                return Err(anyhow::anyhow!(
                    "expected Value::Map for cbor node data, found {:?}",
                    value
                ));
            };

            let mut map = map.into_iter();

            let entries = match map.next() {
                Some((Value::Text(key), Value::Array(entries))) if key == "e" => entries,
                _ => return Err(anyhow::anyhow!("could not parse cbor node data entries")),
            };

            let entries = entries
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<TreeEntry>, _>>()?;

            let left = match map.next() {
                Some((Value::Text(key), value)) if key == "l" => match value {
                    Value::Null => None,
                    value => Some(cid_from_cbor(value)?),
                },
                _ => return Err(anyhow::anyhow!("could not parse cbor node data left")),
            };

            Ok(Self { left, entries })
        }
    }

    impl From<TreeEntry> for Value {
        fn from(value: TreeEntry) -> Self {
            use ciborium::value::Integer;
            Value::Map(vec![
                (Value::Text("k".to_owned()), Value::Bytes(value.key_suffix)),
                (
                    Value::Text("p".to_owned()),
                    Value::Integer(Integer::from(value.prefix_len)),
                ),
                (
                    Value::Text("t".to_owned()),
                    value.tree.map(cid_to_cbor).unwrap_or(Value::Null),
                ),
                (Value::Text("v".to_owned()), cid_to_cbor(value.val)),
            ])
        }
    }

    impl TryFrom<Value> for TreeEntry {
        type Error = anyhow::Error;

        fn try_from(value: Value) -> Result<Self, Self::Error> {
            let Value::Map(map) = value else {
                return Err(anyhow::anyhow!(
                    "expected Value::Map for cbor tree entry, found {:?}",
                    value
                ));
            };

            let mut map = map.into_iter();

            let key_suffix = match map.next() {
                Some((Value::Text(key), Value::Bytes(value))) if key == "k" => value,
                _ => return Err(anyhow::anyhow!("could not parse cbor tree entry key")),
            };

            let prefix_len = match map.next() {
                Some((Value::Text(key), Value::Integer(value))) if key == "p" => {
                    i128::from(value).try_into()?
                }
                _ => return Err(anyhow::anyhow!("could not parse cbor tree entry prefix")),
            };

            let tree = match map.next() {
                Some((Value::Text(key), value)) if key == "t" => match value {
                    Value::Null => None,
                    value => Some(cid_from_cbor(value)?),
                },
                _ => return Err(anyhow::anyhow!("could not parse cbor tree entry tree")),
            };

            let val = match map.next() {
                Some((Value::Text(key), value)) if key == "v" => cid_from_cbor(value)?,
                _ => return Err(anyhow::anyhow!("could not parse cbor tree entry value")),
            };

            Ok(TreeEntry {
                key_suffix,
                prefix_len,
                tree,
                val,
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

            let ciborium_value: ciborium::value::Value =
                ciborium::de::from_reader(&simple_nd_bytes[..]).unwrap();
            println!("ciborium {:?}", ciborium_value);

            let simple_nd = NodeData::try_from(ciborium_value).unwrap();
            println!("simple_nd {:?}", simple_nd);

            let simple_nd_rt_value = Value::from(simple_nd.clone());

            let mut simple_nd_rt_bytes = Vec::new();
            ciborium::ser::into_writer(&simple_nd_rt_value, &mut simple_nd_rt_bytes).unwrap();
            assert_eq!(simple_nd_bytes, simple_nd_rt_bytes);
        }
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
