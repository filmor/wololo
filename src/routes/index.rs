use topcoat::{
    context::{app_context, Cx},
    router::{error::bad_request, page},
    view::{component, view},
    Result,
};

use crate::{providers::ProviderExt, AppState, Error, MacAddress};

#[page("/")]
pub(crate) async fn index(cx: &Cx) -> Result {
    let state = app_context::<AppState>(cx);

    view! {
        <div class="pure-g center-column">
            <div class="pure-u-1">
                <h1>"WoLolo"</h1>
                <p>"Click the 'Wake' button to send a magic packet to a machine."</p>
            </div>
            <div class="pure-u-1">table(state: state)</div>
        </div>
    }
}

#[component]
async fn table(state: &AppState) -> Result {
    view! {
        <table class="pure-table pure-table-horizontal pure-table-striped">
            <thead>
                <tr>
                    <th>"Machine"</th>
                    <th>"MAC Address"</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
                for provider in state.providers.iter() {
                    for name in provider.list_names_sorted().await.map_err(client_error)? {
                        let mac_address = provider.get_mac_address(&name).await.map_err(client_error)?;
                        row(name: &name, mac_address: &mac_address)
                    }
                }
            </tbody>
        </table>
    }
}

#[component]
async fn row(name: &str, mac_address: &MacAddress) -> Result {
    let mac_address = mac_address.to_string();

    view! {
        <tr>
            <td>(name)</td>
            <td><code>(&mac_address)</code></td>
            <td>
                <form method="post" action="/wake" style="margin: auto">
                    <input type="hidden" name="mac_address" value=(&mac_address)>
                    <input type="submit" class="pure-button pure-button-primary" value="Wake">
                </form>
            </td>
        </tr>
    }
}

pub(crate) fn client_error(error: Error) -> topcoat::Error {
    bad_request(error.to_string()).into()
}
