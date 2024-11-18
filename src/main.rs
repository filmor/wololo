mod error;
mod mac_address;
mod providers;
mod routes;

use std::sync::Arc;

use argh::FromArgs;
use error::Error;
use mac_address::MacAddress;
use providers::Provider;

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
}

#[derive(Clone)]
struct AppState {
    providers: Arc<[Box<dyn Provider>]>,
}

unsafe impl Send for AppState {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wololo: Wololo = argh::from_env();

    let bind = format!("{}:{}", wololo.host, wololo.port);

    let mut providers: Vec<Box<dyn Provider>> = Vec::new();

    if !wololo.machine.is_empty() {
        let provider = providers::StaticProvider::from_args(wololo.machine.clone())?;
        providers.push(Box::new(provider));
    }

    let state = AppState { providers: providers.into() };

    // build our application with a single route
    let app = routes::routes(state);

    println!("Listening on: {}", bind);
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
