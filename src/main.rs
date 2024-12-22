mod error;
mod mac_address;
mod providers;
mod routes;

use std::sync::Arc;

use argh::FromArgs;
use error::Error;
use mac_address::MacAddress;
use providers::{FritzBoxProvider, Provider, StaticProvider};
use tokio::signal;

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

    // build our application with a single route
    let app = routes::routes(state);

    log::info!("Listening on: {}", bind);
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
