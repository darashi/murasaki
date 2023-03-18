use bytes::Bytes;

pub struct Client {
    client: reqwest::Client,
    voicevox_url: String,
}

impl Client {
    pub fn new(voicevox_url: &String) -> Self {
        Client {
            client: reqwest::Client::new(),
            voicevox_url: voicevox_url.clone(),
        }
    }

    pub async fn audio_query(&self, speaker: u32, text: &String) -> Result<String, reqwest::Error> {
        let resp = self
            .client
            .post(self.voicevox_url.clone() + "/audio_query")
            .query(&[("speaker", &speaker.to_string()), ("text", &text)])
            .send()
            .await?
            .text()
            .await?;

        Ok(resp)
    }

    pub async fn synthesis(&self, speaker: u32, query: &String) -> Result<Bytes, reqwest::Error> {
        let resp = self
            .client
            .post(self.voicevox_url.clone() + "/synthesis")
            .query(&[("speaker", &speaker.to_string())])
            .body(query.clone())
            .send()
            .await?
            .bytes()
            .await?;

        Ok(resp)
    }
}
