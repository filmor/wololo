mod error;
mod mac_address;
mod providers;
mod routes;

use std::sync::Arc;

use argh::FromArgs;
use error::Error;
use mac_address::MacAddress;
use providers::{FritzBoxProvider, Provider, StaticProvider};

#[derive(FromArgs)]
/// Wake-On-Lan webservice
struct Wololo {
    /// bind host
    #[argh(option, default = "\"127.0.0.1\".to_string()")]
    host: String,

    /// bind port
    #[argh(option, default = "3000")]
    port: u16,

    /// machines
    #[argh(option)]
    machine: Vec<String>,

    /// fritzbox
    #[argh(switch)]
    fritzbox: bool,

    /// fritzbox url
    #[argh(option, default = "\"http://fritz.box:49000\".to_string()")]
    fritzbox_url: String,
}

#[derive(Clone)]
struct AppState {
    providers: Arc<[Box<dyn Provider>]>,
}

impl AppState {
    async fn get_hosts(&self) -> Vec<String> {
        let mut hosts = Vec::new();
        for provider in self.providers.iter() {
            if let Ok(names) = provider.list_names().await {
                hosts.extend(names);
            }
        }

        hosts
    }
}

unsafe impl Send for AppState {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let wololo: Wololo = argh::from_env();

    let bind = format!("{}:{}", wololo.host, wololo.port);

    let mut providers: Vec<Box<dyn Provider>> = Vec::new();

    if !wololo.machine.is_empty() {
        let provider = StaticProvider::from_args(wololo.machine.clone())?;
        providers.push(Box::new(provider));
        log::info!("Static provider enabled for {:?}", wololo.machine);
    }

    if wololo.fritzbox {
        let provider = FritzBoxProvider::new(wololo.fritzbox_url.clone());
        providers.push(Box::new(provider));
        log::info!("FritzBox provider enabled for {}", wololo.fritzbox_url);
    }

    let state = AppState {
        providers: providers.into(),
    };

    log::info!("Available machines: {:?}", state.get_hosts().await);

    // build our application with a single route
    let app = routes::routes(state);

    log::info!("Listening on: {}", bind);
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
