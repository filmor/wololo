use super::Provider;
use crate::{Error, MacAddress};
use futures::{stream, Stream, StreamExt};
use quick_xml::de::from_str;
use reqwest::Client;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

pub struct FritzBoxProvider {
    inner: Arc<FritzBoxProviderInner>,
}

struct FritzBoxProviderInner {
    url: String,
    cached_hosts: RwLock<CachedHosts>,
    refreshing: AtomicBool,
    revision: AtomicU64,
}

impl FritzBoxProvider {
    pub fn new(url: String) -> Self {
        Self {
            inner: Arc::new(FritzBoxProviderInner {
                url,
                cached_hosts: RwLock::new(Default::default()),
                refreshing: AtomicBool::new(false),
                revision: AtomicU64::new(0),
            }),
        }
    }

    async fn get_cached_hosts(&self) -> BTreeMap<String, MacAddress> {
        self.maybe_refresh().await;
        let cached_hosts = self.inner.cached_hosts.read().await;
        cached_hosts.hosts.clone()
    }

    async fn maybe_refresh(&self) {
        let cached_hosts = self.inner.cached_hosts.read().await;
        let is_stale = cached_hosts.timestamp.elapsed() > REFRESH_INTERVAL;
        drop(cached_hosts);

        if !is_stale {
            return;
        }

        if self
            .inner
            .refreshing
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let inner = Arc::clone(&self.inner);
        tokio::spawn(async move {
            if let Err(error) = refresh_hosts(inner.clone()).await {
                log::error!("Failed to fetch hosts from FritzBox: {}", error);
            }
            inner.refreshing.store(false, Ordering::SeqCst);
        });
    }
}

struct CachedHosts {
    timestamp: Instant,
    hosts: BTreeMap<String, MacAddress>,
}

impl CachedHosts {
    fn new() -> Self {
        Self {
            timestamp: Instant::now() - 2 * REFRESH_INTERVAL,
            hosts: Default::default(),
        }
    }

    fn insert(&mut self, host: HostInfo) {
        // TODO: Do not filter active hosts but mark them instead
        if host.active {
            return;
        }

        if let Ok(mac_address) = MacAddress::parse(&host.mac_address) {
            self.hosts.insert(host.host_name, mac_address);
            self.timestamp = Instant::now();
        }
    }
}

impl Default for CachedHosts {
    fn default() -> Self {
        Self::new()
    }
}

async fn get_hosts(fritzbox_url: &str) -> Result<impl Stream<Item = HostInfo> + Send, Error> {
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

    let host_infos = stream::iter(0..total_hosts)
        .map(move |index| {
            let client = client.clone();
            let url = url.clone();
            async move { get_host_info(&client, &url, index).await }
        })
        .buffer_unordered(16)
        .filter_map(|result| async { result.ok() });

    Ok(host_infos)
}

async fn refresh_hosts(inner: Arc<FritzBoxProviderInner>) -> Result<(), Error> {
    let hosts = get_hosts(&inner.url).await?;

    futures::pin_mut!(hosts);
    while let Some(host) = hosts.next().await {
        let mut cached_hosts = inner.cached_hosts.write().await;
        cached_hosts.insert(host);
        inner.revision.fetch_add(1, Ordering::SeqCst);
    }

    Ok(())
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

    fn revision(&self) -> u64 {
        self.inner.revision.load(Ordering::SeqCst)
    }
}
