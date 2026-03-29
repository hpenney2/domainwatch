use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EmbedField<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub inline: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EmbedFooter<'a> {
    pub text: &'a str,
    pub icon_url: Option<&'a str>,
    pub proxy_icon_url: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EmbedAuthor<'a> {
    pub name: &'a str,
    pub url: Option<&'a str>,
    pub icon_url: Option<&'a str>,
    pub proxy_icon_url: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EmebedMedia<'a> {
    pub url: &'a str,
    pub proxy_url: Option<&'a str>,
    pub height: Option<u64>,
    pub width: Option<u64>,
}

impl<'a> EmebedMedia<'a> {
    fn with_url(url: &'a str) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EmbedProvider<'a> {
    pub name: Option<&'a str>,
    pub url: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DiscordEmbed<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub url: Option<&'a str>,

    pub timestamp: Option<&'a str>,
    pub color: Option<u32>,

    pub footer: Option<EmbedFooter<'a>>,
    pub image: Option<EmebedMedia<'a>>,
    pub thumbnail: Option<EmebedMedia<'a>>,
    pub video: Option<EmebedMedia<'a>>,
    pub provider: Option<EmbedProvider<'a>>,
    pub author: Option<EmbedAuthor<'a>>,

    pub fields: Option<Vec<EmbedField<'a>>>,
}

// https://docs.discord.com/developers/resources/webhook#execute-webhook
// some fields are missing because i don't need MOST of them and so i don't see the point in implementing them right now
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExecuteWebhook<'a> {
    pub content: Option<&'a str>,
    pub username: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
    pub tts: Option<bool>,
    pub embeds: Option<Vec<DiscordEmbed<'a>>>,
}
