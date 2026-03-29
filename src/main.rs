use std::fs::File;
use std::{collections::HashMap, env, error::Error, time::Duration};

use rdap::{Domain, RdapClient, RdapRequest};
use serde::{Deserialize, Serialize};
use similar::TextDiff;
use tokio::time::sleep;

mod discord;

const DEFAULT_CONFIG: &str = "config.json";
const ENV_WEBHOOK: &str = "DISCORD_WEBHOOK_URL";

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    responses: HashMap<String, Domain>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!(
            "Usage:\n \
             {name} <domain> [wait time in seconds]\n\n \
             Wait time defaults to 300 seconds and must be a positive number.\n \
             If the {webhook_var} environment variable is set, Discord webhook notifications will be sent to that URL.",
            name = env!("CARGO_BIN_NAME"),
            webhook_var = ENV_WEBHOOK
        );
        return Ok(());
    }

    let domain = args.get(1).expect("must pass in a domain");
    let wait_secs: u64 = args
        .get(2)
        .map(|t| match t.parse::<u64>() {
            Ok(wait) => wait,
            Err(err) => panic!(
                "failed to parse wait time: {}\nmake sure that it is a positive integer number",
                err
            ),
        })
        .unwrap_or(300);

    let webhook_url = match env::var(ENV_WEBHOOK) {
        Ok(url) => Some(url),
        Err(_) => {
            eprintln!(
                "{} variable is unset; webhook notifications will not be sent",
                ENV_WEBHOOK
            );
            None
        }
    };

    // read in config if it exists; otherwise default (no values)
    let config_path = env::var("CONFIG_PATH").unwrap_or(DEFAULT_CONFIG.into());
    let mut config: Config = if let Ok(f) = File::open(DEFAULT_CONFIG) {
        match serde_json::from_reader(f) {
            Ok(c) => c,
            Err(err) => panic!("{:?}", err),
        }
    } else {
        Config::default()
    };

    let client = reqwest::Client::new();
    let rdap = RdapClient::new()?;

    println!("watching domain {} (every {} seconds)", domain, wait_secs);

    loop {
        let mut response: Domain = match rdap
            .query(&RdapRequest::new(rdap::QueryType::Domain, domain))
            .await
            .map_err(|err| format!("error fetching RDAP domain query: {:#?}", err))
            .unwrap()
        {
            rdap::RdapObject::Domain(d) => d,
            unknown => panic!("unexpected response variant {:?}", unknown),
        };

        println!("\n[{}]", jiff::Timestamp::now());
        println!("status:: {}", response.status.join(", "));
        println!("dates:: {:#?}", response.events);

        // strip the update event because it seems to change every time its requested
        // could be server dependent but... who's to say!
        let mut timestamp: String = String::new();
        if let Some(index) = response
            .events
            .iter()
            .position(|value| value.action == "last update of RDAP database")
        {
            timestamp = response.events.remove(index).date;
        }

        if let Some(before) = config.responses.get(domain)
            && !eq_serializable(&before, &response)?
        {
            let before_str = format!("{:#?}", before);
            let after_str = format!("{:#?}", response);

            println!(
                "response changed!\nBEFORE\n{}\n\nAFTER\n{}",
                before_str, after_str
            );

            let diff = TextDiff::from_lines(&before_str, &after_str)
                .unified_diff()
                .header("before", "after")
                .to_string();
            println!("diff::\n{}", diff);

            if let Some(url) = &webhook_url {
                notify_discord(
                    &client,
                    url,
                    "NEW DATA!",
                    domain,
                    &response,
                    &timestamp,
                    Some(&diff),
                )
                .await?;
            }

            config.responses.insert(domain.clone(), response);
            write_config(&config, &config_path)?;
        } else if !config.responses.contains_key(domain) {
            println!("initial response\n{:?}", response);

            if let Some(url) = &webhook_url {
                notify_discord(
                    &client,
                    url,
                    "Initial response",
                    domain,
                    &response,
                    &timestamp,
                    None,
                )
                .await?;
            }

            config.responses.insert(domain.clone(), response);
            write_config(&config, &config_path)?;
        } else {
            println!("~ no differences ~");
        }

        sleep(Duration::from_secs(wait_secs)).await;
    }
}

fn write_config(config: &Config, file_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;
    serde_json::to_writer(file, config)?;
    Ok(())
}

fn eq_serializable(a: &impl Serialize, b: &impl Serialize) -> serde_json::Result<bool> {
    Ok(serde_json::to_string(a)? == serde_json::to_string(b)?)
}

async fn notify_discord(
    client: &reqwest::Client,
    url: &str,
    msg: &str,
    domain: &str,
    data: &Domain,
    timestamp: &str,
    diff: Option<&str>,
) -> Result<(), reqwest::Error> {
    let data_field = format!(
        "```status:: {}\ndates::\n{:#?}```",
        data.status.join(", "),
        data.events
    );
    let diff_field = format!("```diff\n{}```", diff.unwrap_or("[ no diff ]"));

    let embed = discord::DiscordEmbed {
        title: Some(domain),
        description: Some(msg),
        fields: Some(vec![
            discord::EmbedField {
                name: "Data",
                value: &data_field,
                inline: Some(false),
            },
            discord::EmbedField {
                name: "Diff",
                value: &diff_field,
                inline: Some(false),
            },
        ]),
        timestamp: Some(timestamp),

        ..Default::default()
    };

    let body = discord::ExecuteWebhook {
        content: Some("@everyone"),
        embeds: Some(vec![embed]),
        username: Some(env!("CARGO_CRATE_NAME")),
        ..Default::default()
    };

    let response = client.post(url).json(&body).send().await?;

    match response.status() {
        reqwest::StatusCode::NO_CONTENT => Ok(()),
        status => panic!(
            "error sending webhook\n[{}] {}",
            status,
            response.text().await?
        ),
    }
}
