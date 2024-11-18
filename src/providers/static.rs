use super::Provider;
use crate::{Error, MacAddress};
use std::collections::BTreeMap;

pub struct StaticProvider(BTreeMap<String, MacAddress>);

impl StaticProvider {
    pub fn from_args(machine: Vec<String>) -> Result<Self, Error> {
        let mut machines = BTreeMap::new();
        for machine in machine {
            let mut parts = machine.splitn(2, '=');
            let name = parts
                .next()
                .ok_or_else(|| Error::FailedToParseMachineMapping(machine.clone()))?;
            let mac_address = parts
                .next()
                .ok_or_else(|| Error::FailedToParseMachineMapping(machine.clone()))?;

            let mac_address = MacAddress::parse(mac_address)?;

            machines.insert(name.to_string(), mac_address);
        }

        Ok(Self(machines))
    }
}

impl Provider for StaticProvider {
    fn list_names(&self) -> Result<Vec<String>, Error> {
        Ok(self.0.keys().cloned().collect())
    }

    fn get_mac_address(&self, name: &str) -> Result<MacAddress, Error> {
        self.0
            .get(name)
            .cloned()
            .ok_or(Error::FailedToGetMacAddress)
    }
}
