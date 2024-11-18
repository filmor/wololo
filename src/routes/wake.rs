use axum::{extract::State, response::IntoResponse, Form};
use maud::html;
use serde::Deserialize;

use super::Base;
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
        for provider in app_state.providers.iter() {
            if let Ok(addr) = provider.get_mac_address(&machine).await {
                mac_address = Some(addr);
                break;
            }
        }

        if mac_address.is_none() {
            return Err(Error::UnknownMachine);
        }
    }

    let mac_address = mac_address.ok_or(Error::InvalidRequest)?;

    mac_address.send_magic_packet()?;

    Ok(html!{
        (Base)
        meta http-equiv="refresh" content="2; url=/";
        body {
            div class="pure-g center-column" {
                div class="pure-u-1" {
                    h1 { "WoLolo" }
                    p { "Sent magic packet to " (mac_address) }
                }
            }
        }
    })
}
