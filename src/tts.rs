use anyhow::{anyhow, Context};
use log::{info, warn};
use rodio::Decoder;
use std::io::{BufReader, Cursor};

use crate::config::VoiceVoxConfig;
use crate::voicevox;

pub struct TTS {
    vv: voicevox::Client,
    max_retry: u64,
    sink: rodio::Sink,
}

impl TTS {
    pub fn new(sink: rodio::Sink, voicevox_config: &VoiceVoxConfig) -> Self {
        let vv = voicevox::Client::new(&voicevox_config.url);
        let max_retry = voicevox_config.max_retry;
        Self {
            vv,
            max_retry,
            sink,
        }
    }

    pub async fn say(&self, speaker: u32, text: &String) -> anyhow::Result<()> {
        info!("📣 {}", text);

        let query = self
            .vv
            .audio_query(speaker, &text)
            .await
            .context("failed in audio_query")?;

        for _retry in 0..self.max_retry {
            match self.vv.synthesis(speaker, &query).await {
                Err(e) => {
                    warn!("error in synthesis: {}", e);
                }
                Ok(wav) => {
                    let content = Cursor::new(wav);
                    let file = BufReader::new(content);
                    let source = Decoder::new_wav(file);
                    match source {
                        Ok(source) => {
                            self.sink.append(source);
                            return Ok(());
                        }
                        Err(e) => {
                            warn!("failed to decode wav: {}", e);
                        }
                    }
                }
            }
        }

        Err(anyhow!("synthesis retry limit exceeded"))
    }
}
