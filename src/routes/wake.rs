use axum::{extract::State, response::IntoResponse, Form};
use serde::Deserialize;

use crate::{AppState, Error, MacAddress};

#[derive(Deserialize, Default)]
pub struct Wake {
    machine: Option<String>,
    mac_address: Option<String>,
}

pub async fn wake(
    State(app_state): State<AppState>,
    Form(wake): Form<Wake>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut mac_address: Option<MacAddress> = None;

    if let Some(form_addr) = wake.mac_address {
        mac_address = Some(MacAddress::parse(&form_addr)?);
    } else if let Some(machine) = wake.machine {
        if let Some(stored_addr) = app_state.machines.get(&machine) {
            mac_address = Some(stored_addr.clone());
        } else {
            return Err(Error::UnknownMachine);
        }
    }

    let mac_address = mac_address.ok_or(Error::InvalidRequest)?;

    mac_address.send_magic_packet()?;
    Ok(format!("Sent magic packet to {}", mac_address).into_response())
}
