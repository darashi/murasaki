use nostr_sdk::Metadata;

use crate::config::TransformConfig;
use linkify::LinkFinder;
use regex::Regex;

pub struct Transformer {
    config: TransformConfig,
}

impl Transformer {
    pub fn new(config: &TransformConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    fn metadata_to_name(metadata: &Metadata) -> Option<String> {
        if let Some(display_name) = &metadata.display_name {
            if display_name != "" {
                return Some(display_name.clone());
            }
        }
        if let Some(name) = &metadata.name {
            if name != "" {
                return Some(name.clone());
            }
        }

        None
    }

    pub fn transform_reaction(
        &self,
        _event: &nostr_sdk::Event,
        metadata: &Option<Metadata>,
    ) -> String {
        let from: String = metadata
            .as_ref()
            .and_then(|md| Transformer::metadata_to_name(&md))
            .and_then(|name| Some(format!("{}さんから", name)))
            .unwrap_or("".to_string());
        format!("{}リアクション受信。", from)
    }

    pub fn transform_note(&self, event: &nostr_sdk::Event, metadata: &Option<Metadata>) -> String {
        let from = metadata
            .as_ref()
            .and_then(|md| Transformer::metadata_to_name(&md))
            .and_then(|name| Some(format!("{}さん、", name)))
            .unwrap_or("".to_string());

        let text = self.replace_urls(&event.content);
        let text = self.truncate_long(text);
        let text = Self::truncate_nip19(text);

        from + text.as_str()
    }

    fn replace_urls(&self, text: &String) -> String {
        let finder = LinkFinder::new();
        let links: Vec<_> = finder.links(text).collect();
        let mut text = text.clone();
        for link in links {
            text = text.replace(link.as_str(), &self.config.url_alternative_text);
        }
        text
    }

    fn truncate_long(&self, text: String) -> String {
        if text.chars().count() > self.config.max_length {
            text.chars()
                .take(self.config.max_length)
                .collect::<String>()
                + &self.config.ellipsis_text
        } else {
            text
        }
    }

    fn truncate_nip19(text: String) -> String {
        let re =
            Regex::new(r"(nsec|npub|note|nprofile|nevent|nrelay|naddr)1[0-9ac-hj-np-z]+").unwrap();
        re.replace_all(&text, "$1").to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_truncate_nip19() {
        assert_eq!(
            super::Transformer::truncate_nip19(
                "hello npub1xajyg2w6kvslletelz9z94jecdsjmg7jqgrgcn8zvjz78k2sq5fslch3pq test"
                    .to_string()
            ),
            "hello npub test".to_string()
        );
    }
}
