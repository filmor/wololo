use serde::Deserialize;
use topcoat::{
    context::{app_context, Cx},
    router::{content::Form, error::bad_request, page},
    view::view,
    Result,
};

use crate::{AppState, Error, MacAddress};

use super::index::client_error;

#[derive(Deserialize, Default)]
pub struct Wake {
    machine: Option<String>,
    mac_address: Option<String>,
}

#[page(POST "/wake")]
pub(crate) async fn wake(cx: &Cx, Form(wake): Form<Wake>) -> Result {
    let app_state = app_context::<AppState>(cx);
    let mut mac_address: Option<MacAddress> = None;

    if let Some(form_addr) = wake.mac_address {
        mac_address = Some(MacAddress::parse(&form_addr).map_err(client_error)?);
    } else if let Some(machine) = wake.machine {
        for provider in app_state.providers.iter() {
            if let Ok(addr) = provider.get_mac_address(&machine).await {
                mac_address = Some(addr);
                break;
            }
        }

        if mac_address.is_none() {
            return Err(bad_request(Error::UnknownMachine.to_string()).into());
        }
    }

    let mac_address = mac_address.ok_or_else(|| bad_request(Error::InvalidRequest.to_string()))?;

    mac_address.send_magic_packet().map_err(client_error)?;
    let mac_address = mac_address.to_string();

    view! {
        <meta http-equiv="refresh" content="2; url=/">
        <div class="pure-g center-column">
            <div class="pure-u-1">
                <h1>"WoLolo"</h1>
                <p>"Sent magic packet to " (mac_address)</p>
            </div>
        </div>
    }
}
