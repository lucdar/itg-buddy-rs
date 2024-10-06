use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

const APPNAME: &str = "itg-buddy";
const CONFIGNAME: &str = "config";

macro_rules! prompt_field {
    ($config:ident.$field:ident) => {
        let mut buf: String = "".into();
        print!(
            "Input your {}: ",
            stringify!($field).replace("_", " ").yellow().bold()
        );
        io::stdout().flush().context("Failed to flush stdout")?;
        io::stdin()
            .read_line(&mut buf)
            .context("Failed to read line")?;
        trim_newline(&mut buf);
        $config.$field = buf.into();
    };
}

#[derive(Serialize, Deserialize, Default)]
pub struct ITGBuddyConfig {
    pub discord_key: String,
    pub add_song_channel_id: String,
}
impl ITGBuddyConfig {
    pub fn new() -> Result<ITGBuddyConfig> {
        let mut config = ITGBuddyConfig::default();
        prompt_field!(config.discord_key);
        Ok(config)
    }
    pub fn store(&self) -> Result<()> {
        confy::store(APPNAME, CONFIGNAME, self).context("Failed to store config")?;
        confy::get_configuration_file_path(APPNAME, CONFIGNAME)?
            .to_str()
            .inspect(|s| println!("Saved config to {}", s.underline().bold()));
        Ok(())
    }
    pub fn load() -> Result<ITGBuddyConfig> {
        confy::load(APPNAME, CONFIGNAME).context("Failed to load config file")
    }
}
fn trim_newline(s: &mut String) {
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
}
