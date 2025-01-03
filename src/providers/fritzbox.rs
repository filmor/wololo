use super::Provider;
use crate::{Error, MacAddress};
use futures::{stream, StreamExt};
use quick_xml::de::from_str;
use reqwest::Client;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

pub struct FritzBoxProvider {
    url: String,
    cached_hosts: RwLock<CachedHosts>,
}

impl FritzBoxProvider {
    pub fn new(url: String) -> Self {
        Self {
            url,
            cached_hosts: RwLock::new(Default::default()),
        }
    }

    async fn get_cached_hosts(&self) -> BTreeMap<String, MacAddress> {
        let cached_hosts = self.cached_hosts.read().await;
        if cached_hosts.timestamp.elapsed() > REFRESH_INTERVAL {
            drop(cached_hosts);

            match get_hosts(&self.url).await {
                Ok(new_hosts) => {
                    let mut cached_hosts = self.cached_hosts.write().await;
                    *cached_hosts = CachedHosts::new(new_hosts.into_iter());
                }
                Err(e) => {
                    log::error!("Failed to fetch hosts from FritzBox: {}", e);
                }
            }
        }

        let cached_hosts = self.cached_hosts.read().await;
        cached_hosts.hosts.clone()
    }
}

struct CachedHosts {
    timestamp: Instant,
    hosts: BTreeMap<String, MacAddress>,
}

impl CachedHosts {
    fn new(hosts: impl Iterator<Item = HostInfo>) -> Self {
        Self {
            timestamp: Instant::now(),
            hosts: hosts
                .filter_map(|host| {
                    if host.active {
                        return None;
                    }

                    MacAddress::parse(&host.mac_address)
                        .ok()
                        .map(|m| (host.host_name, m))
                })
                .collect(),
        }
    }
}

impl Default for CachedHosts {
    fn default() -> Self {
        Self {
            timestamp: Instant::now() - 2 * REFRESH_INTERVAL,
            hosts: Default::default(),
        }
    }
}

async fn get_hosts(fritzbox_url: &str) -> Result<Vec<HostInfo>, Error> {
    log::info!("Fetching hosts from FritzBox {}", fritzbox_url);
    let client = Client::new();

    // SOAP headers and body for GetHostNumberOfEntries
    let soap_body = r#"<?xml version="1.0"?>
        <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
                    s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
            <s:Body>
                <u:GetHostNumberOfEntries xmlns:u="urn:dslforum-org:service:Hosts:1" />
            </s:Body>
        </s:Envelope>"#;

    let url = format!("{}/upnp/control/hosts", fritzbox_url);

    // Send request to FritzBox
    let response = client
        .post(&url)
        // .basic_auth(username, Some(password))
        .header("Content-Type", "text/xml; charset=utf-8")
        .header(
            "SOAPAction",
            "urn:dslforum-org:service:Hosts:1#GetHostNumberOfEntries",
        )
        .body(soap_body)
        .send()
        .await?;

    // Parse response to get the total number of hosts
    let response_text = response.text().await?;

    // TODO: Parse the XML response properly
    let total_hosts: u32 = response_text
        .split("<NewHostNumberOfEntries>")
        .nth(1)
        .and_then(|s| s.split("</NewHostNumberOfEntries>").next())
        .and_then(|s| s.parse().ok())
        .ok_or(Error::FailedToListNames)?;

    log::info!("Total hosts: {}", total_hosts);

    let host_infos: Vec<_> = stream::iter(0..total_hosts)
        .map(|index| get_host_info(&client, &url, index))
        .buffer_unordered(16)
        .filter_map(|result| async { result.ok() })
        .collect()
        .await;

    log::info!("Fetched {} hosts", host_infos.len());

    Ok(host_infos)
}

#[derive(Debug, Deserialize, Clone)]
struct Envelope {
    #[serde(rename = "Body")]
    body: Body,
}

#[derive(Debug, Deserialize, Clone)]
struct Body {
    #[serde(rename = "GetGenericHostEntryResponse")]
    response: HostInfo,
}

#[derive(Debug, Deserialize, Clone)]
struct HostInfo {
    #[serde(rename = "NewMACAddress")]
    mac_address: String,
    #[serde(rename = "NewHostName")]
    host_name: String,
    // #[serde(rename = "NewIPAddress")]
    // ip_address: String,
    // #[serde(rename = "NewInterfaceType")]
    // interface_type: String,
    #[serde(rename = "NewActive")]
    active: bool,
}

async fn get_host_info(client: &Client, url: &str, index: u32) -> Result<HostInfo, Error> {
    let soap_body = format!(
        r#"<?xml version="1.0"?>
            <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
                        s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
                <s:Body>
                    <u:GetGenericHostEntry xmlns:u="urn:dslforum-org:service:Hosts:1">
                        <NewIndex>{}</NewIndex>
                    </u:GetGenericHostEntry>
                </s:Body>
            </s:Envelope>"#,
        index
    );

    let response = client
        .post(url)
        .header("Content-Type", "text/xml; charset=utf-8")
        .header(
            "SOAPAction",
            "urn:dslforum-org:service:Hosts:1#GetGenericHostEntry",
        )
        .body(soap_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    log::debug!("Response: {}", response_text);
    Ok(from_str::<Envelope>(&response_text)?.body.response)
}

#[async_trait::async_trait]
impl Provider for FritzBoxProvider {
    async fn list_names(&self) -> Result<Vec<String>, Error> {
        let hosts = self.get_cached_hosts().await;
        Ok(hosts.keys().cloned().collect())
    }

    async fn get_mac_address(&self, name: &str) -> Result<MacAddress, Error> {
        let hosts = self.get_cached_hosts().await;
        hosts.get(name).cloned().ok_or(Error::UnknownMachine)
    }
}
