use anyhow::{anyhow, Context};
use clap::Parser;
use log::{info, warn};
use murasaki::config::Config;
use murasaki::transformer::Transformer;
use murasaki::tts::TTS;
use nostr_sdk::prelude::FromPkStr;
use nostr_sdk::secp256k1::XOnlyPublicKey;
use rodio::{OutputStream, Sink};
use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs};

use nostr_sdk::{self, Client, Metadata};
use nostr_sdk::{Filter, Keys, Kind, RelayPoolNotification};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,
}

async fn fetch_metadata(
    nostr_client: &Client,
    pubkey: &XOnlyPublicKey,
) -> anyhow::Result<Metadata> {
    let timeout = Duration::from_secs(10);
    let metadata_subscription = Filter::new()
        .kinds(vec![Kind::Metadata])
        .author(pubkey.to_string());
    let events = nostr_client
        .get_events_of(vec![metadata_subscription], Some(timeout))
        .await?;
    for event in events {
        if event.kind != Kind::Metadata {
            continue;
        }
        if event.pubkey != *pubkey {
            continue;
        }

        return Ok(Metadata::from_json(&event.content)?);
    }

    Err(anyhow!("no metadata found"))
}

struct Murasaki {
    config: Config,
    tts: TTS,
    metadata_db: murasaki::metadata::Cache,
    nostr_client: nostr_sdk::Client,
    text_transformer: Transformer,
    following_mode: bool,
    pubkey_provided: bool,
}

impl Murasaki {
    fn new(config: Config, sink: Sink) -> anyhow::Result<Self> {
        let mut following_mode = false;
        let mut pubkey_provided = false;
        let my_keys: Keys = if let Some(pubkey) = &config.nostr.pubkey {
            info!("using pubkey: {}", pubkey);
            following_mode = true;
            pubkey_provided = true;
            Keys::from_pk_str(&pubkey).context("failed to parse pubkey")?
        } else {
            info!("pubkey is not defined");
            Keys::generate()
        };
        let nostr_client = nostr_sdk::Client::new(&my_keys);

        let tts = TTS::new(sink, &config.voicevox);

        let text_transformer = Transformer::new(&config.transform);

        Ok(Self {
            config,
            tts,
            metadata_db: murasaki::metadata::Cache::new(Duration::from_secs(5 * 60)),
            nostr_client,
            text_transformer,
            following_mode,
            pubkey_provided,
        })
    }

    async fn connect(&self) -> anyhow::Result<()> {
        for relay in &self.config.nostr.relays {
            info!("adding relay: {}", relay);
            self.nostr_client
                .add_relay(relay, None)
                .await
                .with_context(|| format!("failed to add relay `{}`", relay))?;
        }
        self.nostr_client.connect().await;
        Ok(())
    }

    async fn subscribe(&self) -> anyhow::Result<()> {
        let mut filters: Vec<Filter> = vec![];

        if self.pubkey_provided {
            let timeout = Duration::from_secs(10);
            let contact = self.nostr_client.get_contact_list(Some(timeout)).await?;
            let pks: HashSet<_> = contact.iter().map(|c| c.pk).collect(); // dedup
            info!("{} followers found!", pks.len());

            let notes_filter = if self.following_mode {
                Filter::new()
                    .limit(0)
                    .kinds(vec![Kind::TextNote, Kind::ContactList])
                    .authors(pks.into_iter().map(|pk| pk.to_string()).collect())
            } else {
                Filter::new().limit(0).kinds(vec![Kind::TextNote])
            };
            filters.push(notes_filter);

            let mention_filter = Filter::new()
                .limit(0)
                .kinds(vec![Kind::TextNote, Kind::Reaction, Kind::ContactList])
                .pubkey(self.nostr_client.keys().public_key());
            filters.push(mention_filter);
        } else {
            let notes_filter = Filter::new().limit(0).kinds(vec![Kind::TextNote]);
            filters.push(notes_filter);
        };

        self.nostr_client.subscribe(filters).await;
        Ok(())
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        self.connect().await?;
        self.subscribe().await?;

        let connect_message = if self.following_mode {
            "接続しました。フォロイングモードです。"
        } else {
            "接続しました。ユニバースモードです。"
        };
        self.tts
            .say(self.config.speaker, &connect_message.to_string())
            .await?;

        loop {
            let mut notifications = self.nostr_client.notifications();
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    if let Err(e) = self.handle_event(&event).await {
                        warn!("failed to handle event: {}", e);
                    }
                }
            }
        }
    }

    async fn handle_event(&mut self, event: &nostr_sdk::Event) -> anyhow::Result<()> {
        match event.kind {
            Kind::TextNote => {
                self.handle_textnote(&event).await?;
            }
            Kind::ContactList => {
                if event.pubkey == self.nostr_client.keys().public_key() {
                    info!("contact list updated");
                    if self.following_mode {
                        self.subscribe().await?;
                    }
                }
            }
            Kind::Reaction => {
                self.handle_reaction(&event).await?;
            }
            _ => return Ok(()),
        }
        Ok(())
    }

    fn is_old(&self, event: &nostr_sdk::Event) -> bool {
        let created_at = UNIX_EPOCH + Duration::from_secs(event.created_at.as_u64());
        if let Ok(age) = SystemTime::now().duration_since(created_at) {
            if age.as_secs() > self.config.nostr.old_threshold_seconds {
                return true;
            }
        }

        false
    }

    async fn get_metadata_with_cache(&mut self, pubkey: &XOnlyPublicKey) -> Option<Metadata> {
        if let Some(metadata) = self.metadata_db.get(&pubkey) {
            Some(metadata.to_owned())
        } else {
            let metadata = fetch_metadata(&self.nostr_client, &pubkey)
                .await
                .with_context(|| format!("failed to fetch metadata for {:?}", pubkey));
            match metadata {
                Ok(metadata) => {
                    self.metadata_db.insert(*pubkey, metadata.clone());
                    Some(metadata)
                }
                Err(e) => {
                    warn!("failed to fetch metadata: {}", e);
                    None
                }
            }
        }
    }

    async fn handle_textnote(&mut self, event: &nostr_sdk::Event) -> anyhow::Result<()> {
        if self.is_old(event) {
            warn!("skipping old event {:?}", event);
            return Ok(());
        }
        let md = self.get_metadata_with_cache(&event.pubkey).await;
        let text = self.text_transformer.transform_note(&event, &md);
        self.tts.say(self.config.speaker, &text).await
    }

    async fn handle_reaction(&mut self, event: &nostr_sdk::Event) -> anyhow::Result<()> {
        if self.is_old(event) {
            warn!("skipping old event {:?}", event);
            return Ok(());
        }
        let metadata = self.get_metadata_with_cache(&event.pubkey).await;
        info!("reaction received {}", event.content);
        let text = self.text_transformer.transform_reaction(&event, &metadata);
        self.tts.say(self.config.speaker, &text).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    let args = Args::parse();

    let contents = fs::read_to_string(&args.config)
        .with_context(|| format!("could not read file `{}`", &args.config))?;
    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("could not parse file `{}`", &args.config))?;

    let (_stream, stream_handle) =
        OutputStream::try_default().context("failed to open output tream")?;
    let sink = rodio::Sink::try_new(&stream_handle).context("failed to create sink")?;

    let mut murasaki = Murasaki::new(config, sink)?;
    murasaki.run().await
}
