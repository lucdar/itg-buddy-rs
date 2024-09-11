use anyhow::{Error, Result};
use config::ITGBuddyConfig;
use poise::serenity_prelude as serenity;

type Context<'a> = poise::Context<'a, Data, Error>;
struct Data {} // User data, which is stored and accessible in all command invocations

// Replys to the command, optionally with a supplied message
#[poise::command(slash_command, prefix_command)]
async fn ping(
    ctx: Context<'_>,
    #[description = "Message to respond with"] msg: Option<String>,
) -> Result<()> {
    let response = format!("Pong! {}", msg.unwrap_or("".into()));
    ctx.reply(response).await?;
    Ok(())
}

// Spawn poise boxes to register or deregister slash commands
#[poise::command(prefix_command, slash_command, owners_only)]
async fn register(ctx: Context<'_>) -> Result<()> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().nth(1) == Some("setup".into()) {
        let config = ITGBuddyConfig::new()?;
        std::process::exit(0);
    };

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::union(
        serenity::GatewayIntents::non_privileged(),
        serenity::GatewayIntents::MESSAGE_CONTENT,
    );

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), register()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    Ok(client.unwrap().start().await?)
}

mod config {
    use anyhow::{Context, Result};
    use colored::Colorize;
    use serde::{Deserialize, Serialize};
    use std::io::{self, Write};
    #[derive(Serialize, Deserialize, Default)]
    pub struct ITGBuddyConfig {
        discord_key: String,
        itg_cli_dir: String,
    }
    impl ITGBuddyConfig {
        pub fn new() -> Result<ITGBuddyConfig> {
            let mut config = ITGBuddyConfig::default();
            // discord_key
            print!("Input your {}: ", "discord key".yellow().bold());
            io::stdout().flush().context("Failed to flush stdout")?;
            io::stdin()
                .read_line(&mut config.discord_key)
                .context("Failed to read line")?;
            trim_newline(&mut config.discord_key);
            // itg_cli_dir
            print!("Input your {}: ", "itg-cli install path".yellow().bold());
            io::stdout().flush().context("Failed to flush stdout")?;
            io::stdin()
                .read_line(&mut config.itg_cli_dir)
                .context("Failed to read line")?;
            trim_newline(&mut config.itg_cli_dir);

            Ok(config)
        }
    }
    fn trim_newline(s: &mut String) {
        while s.ends_with('\n') || s.ends_with('\r') {
            s.pop();
        }
    }
}
