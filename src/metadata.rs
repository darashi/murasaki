use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use log::info;
use nostr_sdk::{secp256k1::XOnlyPublicKey, Metadata};

pub struct Cache {
    metadata: HashMap<XOnlyPublicKey, Metadata>,
    fetched_at: HashMap<XOnlyPublicKey, SystemTime>,
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl: Duration) -> Self {
        Cache {
            metadata: HashMap::new(),
            fetched_at: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&mut self, key: &XOnlyPublicKey) -> Option<&Metadata> {
        self.purge_old();
        self.metadata.get(key)
    }

    pub fn insert(&mut self, key: XOnlyPublicKey, value: Metadata) {
        self.purge_old();
        self.metadata.insert(key, value);
        self.fetched_at.insert(key, SystemTime::now());
    }

    pub fn purge_old(&mut self) {
        let now = SystemTime::now();
        let threshold = now - self.ttl;

        let keys_to_delete = self
            .fetched_at
            .iter()
            .filter(|(_, fetched_at)| fetched_at < &&threshold)
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        if !keys_to_delete.is_empty() {
            info!("purging {} old metadata", keys_to_delete.len());
        }

        for key in keys_to_delete {
            self.metadata.remove(&key);
            self.fetched_at.remove(&key);
        }
    }
}
