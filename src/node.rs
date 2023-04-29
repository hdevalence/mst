use libipld::{Cid, DagCbor};

#[derive(Clone, Debug, DagCbor, PartialEq, Eq)]
pub struct TreeEntry {
    #[ipld(rename = "p")]
    prefix_len: u64,
    #[ipld(rename = "k")]
    key_suffix: Box<[u8]>,
    #[ipld(rename = "v")]
    val: Cid,
    #[ipld(rename = "t")]
    tree: Option<Cid>,
}

#[derive(Clone, Debug, DagCbor, PartialEq, Eq)]
pub struct NodeData {
    #[ipld(rename = "l")]
    left: Option<Cid>,
    #[ipld(rename = "e")]
    entries: Vec<TreeEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_test_vector() {
        let hex_simple_nd = "a2616581a4616b5820636f6d2e6578616d706c652e7265636f72642f336a716663717a6d33666f326a6170006174f66176d82a582500017112209d156bc3f3a520066252c708a9361fd3d089223842500e3713d404fdccb33cef616cf6";
        let simple_nd_bytes = hex::decode(&hex_simple_nd).unwrap();

        use libipld::{
            cbor::DagCborCodec,
            codec::{Decode, Encode},
        };
        use std::io::Cursor;

        let simple_nd = NodeData::decode(DagCborCodec, &mut Cursor::new(&simple_nd_bytes)).unwrap();
        //dbg!(&simple_nd);

        let mut buf = Cursor::new(Vec::new());
        simple_nd.encode(DagCborCodec, &mut buf).unwrap();
        let simple_nd_bytes2 = buf.into_inner();

        assert_eq!(simple_nd_bytes, simple_nd_bytes2);
    }
}
