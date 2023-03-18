use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct VoiceVoxConfig {
    pub url: String,
    pub max_retry: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransformConfig {
    pub url_alternative_text: String,
    pub max_length: usize,
    pub ellipsis_text: String,
    pub read_name: bool,
}

#[derive(Deserialize, Debug)]
pub struct NostrConfig {
    pub relays: Vec<String>,
    pub old_threshold_seconds: u64,
    pub pubkey: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub voicevox: VoiceVoxConfig,
    pub nostr: NostrConfig,
    pub speaker: u32,
    pub transform: TransformConfig,
}
