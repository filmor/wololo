mod error;
mod mac_address;
mod providers;
mod routes;

use std::sync::Arc;

use argh::FromArgs;
use error::Error;
use mac_address::MacAddress;
use providers::{FritzBoxProvider, Provider, StaticProvider};
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::Router,
};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let wololo: Wololo = argh::from_env();

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

    let assets = match std::env::var_os("WOLOLO_ASSET_DIR") {
        Some(directory) => AssetBundle::load_dir(directory)?,
        None => AssetBundle::load()?,
    };

    let app = Router::builder()
        .app_context(state)
        .assets(assets)
        .page(routes::index::index)
        .page(routes::wake::wake)
        .discover_routes()
        .layout(routes::util::base)
        .build();

    // Keep the existing command-line interface while using Topcoat's server.
    unsafe {
        std::env::set_var("HOST", wololo.host);
        std::env::set_var("PORT", wololo.port.to_string());
    }
    topcoat::start(app).await?;

    Ok(())
}
