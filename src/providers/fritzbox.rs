use super::Provider;
use crate::{Error, MacAddress};

pub struct FritzBoxProvider {
    url: String,
    username: String,
    password: String,
}

impl FritzBoxProvider {
    pub fn new(url: String, username: String, password: String) -> Self {
        Self {
            url,
            username,
            password,
        }
    }
}

impl Provider for FritzBoxProvider {
    fn list_names(&self) -> Result<Vec<String>, Error> {
        unimplemented!()
    }

    fn get_mac_address(&self, name: &str) -> Result<MacAddress, Error> {
        unimplemented!()
    }
}
