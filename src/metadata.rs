use std::collections::HashMap;

use nostr_sdk::{secp256k1::XOnlyPublicKey, Metadata};

pub struct Database {
    metadata: HashMap<XOnlyPublicKey, Metadata>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            metadata: HashMap::new(),
        }
    }

    pub fn get(&self, key: &XOnlyPublicKey) -> Option<&Metadata> {
        self.metadata.get(key)
    }

    pub fn insert(&mut self, key: XOnlyPublicKey, value: Metadata) {
        self.metadata.insert(key, value);
        // TODO purge old metadata
    }
}
