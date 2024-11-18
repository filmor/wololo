use crate::{Error, MacAddress};

mod fritzbox;
mod r#static;

pub trait Provider: Send + Sync {
    fn list_names(&self) -> Result<Vec<String>, Error>;
    fn get_mac_address(&self, name: &str) -> Result<MacAddress, Error>;
}

pub use fritzbox::FritzBoxProvider;
pub use r#static::StaticProvider;
