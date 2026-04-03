use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity::*};

const DISCORD_CLIENT_ID: &str = "1489624876909330452";

macro_rules! default_activity {
    ($v:expr) => {
        Activity::new()
            .details(format!("Minecraft: Rust Edition — {}", $v))
            .assets(
                Assets::new()
                    .large_image("green-apple")
                    .large_text("Minecraft client but in rust"),
            )
    };
}

#[derive(PartialEq)]
pub enum PresenceState {
    Loading,
    InMenu,
    Multiplayer,
}

pub struct DiscordPresence {
    client: DiscordIpcClient,
    state: PresenceState,
}

impl DiscordPresence {
    pub fn start(version: impl AsRef<str>) -> Result<Self, Box<dyn std::error::Error>> {
        let version = version.as_ref();

        let mut client = DiscordIpcClient::new(DISCORD_CLIENT_ID);
        client.connect()?;

        let payload = default_activity!(version).state("Starting...");

        client.set_activity(payload)?;

        Ok(Self {
            client,
            state: PresenceState::Loading,
        })
    }

    pub fn set_in_menu(
        &mut self,
        version: impl AsRef<str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.state == PresenceState::InMenu {
            return Ok(());
        }

        let version = version.as_ref();

        let payload = default_activity!(version).state("In the menu");

        self.state = PresenceState::InMenu;
        self.set_activity(payload)?;

        Ok(())
    }

    pub fn playing_multiplayer(
        &mut self,
        version: impl AsRef<str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.state == PresenceState::Multiplayer {
            return Ok(());
        }

        let version = version.as_ref();

        let payload = default_activity!(version).state("In a server");

        self.state = PresenceState::Multiplayer;
        self.set_activity(payload)?;

        Ok(())
    }

    fn set_activity(&mut self, payload: Activity) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(discord_rich_presence::error::Error::NotConnected) =
            self.client.set_activity(payload.clone())
        {
            self.client.connect()?;
            self.client.set_activity(payload)?;
        }
        Ok(())
    }
}

impl Drop for DiscordPresence {
    fn drop(&mut self) {
        let _ = self.client.clear_activity();
        let _ = self.client.close();
    }
}
