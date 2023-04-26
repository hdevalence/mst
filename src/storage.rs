use std::collections::BTreeMap;



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
