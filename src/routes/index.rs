use axum::extract::State;
use maud::{html, Markup};

use super::Base;
use crate::{Error, AppState, MacAddress};

pub async fn index(State(state): State<AppState>) -> Result<Markup, Error> {
    Ok(html! {
        (Base)
        body {
            div class="pure-g center-column" {
                div class="pure-u-1" {
                    h1 { "WoLolo" }
                    p { "Click the 'Wake' button to send a magic packet to a machine." }
                }
                div class="pure-u-1" {
                    (table(&state).await?)
                }
            }
        }
    })
}

async fn table(state: &AppState) -> Result<Markup, Error> {
    Ok(html! {
        table class="pure-table pure-table-horizontal pure-table-striped" {
            thead {
                tr {
                    th { "Machine" }
                    th { "MAC Address" }
                    th { "" }
                }
            }
            tbody {
                @for provider in state.providers.into_iter() {
                    @for name in provider.list_names().await? {
                        @let mac_address = provider.get_mac_address(&name).await?;
                        (row(&name, &mac_address))
                    }
                }
            }
        }
    })
}

fn row(name: &str, mac_address: &MacAddress) -> Markup {
    html! {
        tr {
            td { (name) }
            td { code { (mac_address) } }
            td {
                form method="post" action="/wake" style="margin: auto" {
                    input type="hidden" name="mac_address" value=(mac_address);
                    input type="submit" class="pure-button pure-button-primary" value="Wake";
                }
            }
        }
    }
}
