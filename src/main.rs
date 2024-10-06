use anyhow::{Error, Result};
use config::ITGBuddyConfig;
use indoc::formatdoc;
use poise::serenity_prelude as serenity;
use serenity::async_trait;

mod config;

type Context<'a> = poise::Context<'a, Data, Error>;
struct Data {} // User data, which is stored and accessible in all command invocations

struct AddSongHandler {
    watched_channel_id: String,
}
#[async_trait]
impl serenity::EventHandler for AddSongHandler {
    async fn message(&self, ctx: serenity::Context, msg: serenity::Message) {
        if msg.channel_id.to_string() != self.watched_channel_id || msg.attachments.is_empty() {
            return;
        }
        for zip in msg
            .attachments
            .iter()
            .filter(|x| x.filename.ends_with(".zip"))
        {
            let response = formatdoc! {"
                ## Calling `add-song`
                **url**: {url}
                **singles**: 
                **cache**:", 
                url=zip.url
            };
            println!("Calling add-song: {}", zip.filename);
            let _ = msg
                .reply(&ctx.http, response)
                .await
                .inspect_err(|e| eprintln!("failed to send message {e}"));
            // TODO: Call itg_cli add_song function
        }
    }
}

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
        ITGBuddyConfig::new()?.store()?;
        std::process::exit(0);
    };

    let cfg = ITGBuddyConfig::load()?;
    let token = cfg.discord_key;

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
        .event_handler(AddSongHandler {
            watched_channel_id: cfg.add_song_channel_id,
        })
        .await;
    Ok(client.unwrap().start().await?)
}
