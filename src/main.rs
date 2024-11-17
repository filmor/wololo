mod error;
mod mac_address;
mod routes;

use std::collections::BTreeMap;

use argh::FromArgs;
use error::Error;
use mac_address::MacAddress;

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

#[derive(Debug, Clone)]
struct AppState {
    machines: BTreeMap<String, MacAddress>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wololo: Wololo = argh::from_env();

    let bind = format!("{}:{}", wololo.host, wololo.port);

    let mut machines = BTreeMap::new();
    for machine in wololo.machine {
        let mut parts = machine.splitn(2, '=');
        let name = parts
            .next()
            .ok_or_else(|| Error::FailedToParseMachine(machine.clone()))?;
        let mac_address = parts
            .next()
            .ok_or_else(|| Error::FailedToParseMachine(machine.clone()))?;

        let mac_address = MacAddress::parse(mac_address)?;

        machines.insert(name.to_string(), mac_address);
    }

    let state = AppState { machines };

    // build our application with a single route
    let app = routes::routes(state);

    println!("Listening on: {}", bind);
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
