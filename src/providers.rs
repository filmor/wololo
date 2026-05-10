use crate::{Error, MacAddress};

mod fritzbox;
mod r#static;

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn list_names(&self) -> Result<Vec<String>, Error>;
    async fn get_mac_address(&self, name: &str) -> Result<MacAddress, Error>;
}

// Extension method for provider to get names sorted by name
pub trait ProviderExt: Provider {
    async fn list_names_sorted(&self) -> Result<Vec<String>, Error>;
}

impl<T: Provider + ?Sized> ProviderExt for T {
    async fn list_names_sorted(&self) -> Result<Vec<String>, Error> {
        let mut names = self.list_names().await?;
        names.sort_by_key(|a| a.to_lowercase());
        Ok(names)
    }
}


pub use fritzbox::FritzBoxProvider;
pub use r#static::StaticProvider;
