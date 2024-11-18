use axum::extract::State;
use maud::{html, Markup};

use crate::{AppState, MacAddress};

pub async fn index(State(state): State<AppState>) -> Markup {
    html! {
        link rel="stylesheet" type="text/css" href="/style.css";
        h1 { "WoLolo" }
        table {
            thead {
                tr {
                    th { "Machine" }
                    th { "MAC Address" }
                    th { "" }
                }
            }
            tbody {
                @for provider in state.providers.into_iter() {
                    @for name in provider.list_names().unwrap() {
                        @let mac_address = provider.get_mac_address(&name).unwrap();
                        (row(&name, &mac_address))
                    }
                }
            }
        }
    }
}

fn row(name: &str, mac_address: &MacAddress) -> Markup {
    html! {
        tr {
            td { (name) }
            td { (mac_address) }
            td {
                form method="post" action="/wake" {
                    input type="hidden" name="machine" value=(name);
                    input type="submit" value="Wake";
                }
            }
        }
    }
}
