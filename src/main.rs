use anyhow::{Error, Result};
use config::ITGBuddyConfig;
use poise::serenity_prelude as serenity;
use serenity::async_trait;

mod config;
mod itg_endpoint;
use itg_endpoint::ItgEndpoint;

type Context<'a> = poise::Context<'a, Data, Error>;
struct Data {} // User data, which is stored and accessible in all command invocations

struct AddSongHandler {
    watched_channel_id: String,
}
#[async_trait]
impl serenity::EventHandler for AddSongHandler {
    async fn message(&self, ctx: serenity::Context, msg: serenity::Message) {
        let wrong_channel = msg.channel_id.to_string() != self.watched_channel_id;
        let no_attachments = msg.attachments.is_empty();
        if wrong_channel || no_attachments {
            return;
        }
        for zip in msg
            .attachments
            .iter()
            .filter(|x| x.filename.ends_with(".zip"))
        {
            let endpoint = match ItgEndpoint::new("http://localhost:50051").await {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    println!("Error connecting to itg endpoint: {e}");
                    let e_str = format!("Couldn't connect to itg endpoint: ```{e}```");
                    let _ = msg.reply(&ctx.http, e_str).await;
                    return;
                }
            };
            match endpoint.add_song(&zip.url, true).await {
                Ok(result) => {
                    let succ_msg =
                        format!("Added {} to {}.", result.added_song, result.destination);
                    let _ = msg.reply(&ctx.http, succ_msg).await;
                }
                Err(e) => {
                    println!("Error adding song: {e}");
                    let _ = msg.reply(&ctx.http, "Error adding song.").await;
                }
            }
        }
    }
}

// Calls `itg-cli add-song` on the supplied URL
#[poise::command(slash_command, prefix_command)]
async fn add_song(
    ctx: Context<'_>,
    #[description = "Link to song to add"] url: String,
) -> Result<()> {
    let result = ItgEndpoint::new("http://localhost:50051")
        .await?
        .add_song(&url, true)
        .await?;
    let s = format!("Added {} to {}.", result.added_song, result.destination);
    ctx.reply(s).await?;
    Ok(())
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
