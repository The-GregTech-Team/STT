use async_minecraft_ping::ConnectionConfig;
use async_trait::async_trait;

use super::{Service, ServiceError, ServiceResult};

#[derive(Debug)]
pub enum MinecraftServiceDetectionMode {
    OnlinePlayers,
    PlayerList,
}

impl Default for MinecraftServiceDetectionMode {
    fn default() -> Self {
        MinecraftServiceDetectionMode::OnlinePlayers
    }
}

pub struct MinecraftService {
    /// Minecraft server address
    address: String,

    /// Minecraft server port
    port: u16,

    /// Decide whether the Minecraft server is in use based on online players field or player list field that returns.
    /// Usually they should be same in vanilla, but it may vary in thirdparty servers whose online player count returns a custom number.
    detection_mode: MinecraftServiceDetectionMode,
}

#[async_trait]
impl Service for MinecraftService {
    async fn start(&self) {
        todo!()
    }

    async fn stop(&self) {
        todo!()
    }

    async fn busy(&self) -> ServiceResult<bool> {
        let config = ConnectionConfig::build(&self.address).with_port(self.port);
        let status = config
            .connect()
            .await
            .map_err(|e| ServiceError::External(e.into()))?
            .status()
            .await
            .map_err(|e| ServiceError::External(e.into()))?;
        Ok(match self.detection_mode {
            MinecraftServiceDetectionMode::OnlinePlayers => status.status.players.online > 0,
            MinecraftServiceDetectionMode::PlayerList => status
                .status
                .players
                .sample
                .map(|x| !x.is_empty())
                .unwrap_or_default(),
        })
    }
}
