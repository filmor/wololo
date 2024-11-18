use crate::{Error, MacAddress};

mod fritzbox;
mod r#static;

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn list_names(&self) -> Result<Vec<String>, Error>;
    async fn get_mac_address(&self, name: &str) -> Result<MacAddress, Error>;
}

pub use fritzbox::FritzBoxProvider;
pub use r#static::StaticProvider;
