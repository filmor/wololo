use topcoat::{asset::asset, router::layout, view::view, Result};

const PURE_CSS: topcoat::asset::Asset = asset!("assets/pure-min.css");
const MACHINES_JS: topcoat::asset::Asset = asset!("assets/machines.js");

#[layout("/")]
pub(crate) async fn base(slot: Result) -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8">
                <title>"WoLolo"</title>
                <link rel="stylesheet" type="text/css" href=(PURE_CSS)>
                <script type="module" src=(MACHINES_JS)></script>
                <style>
                    ".center-column { max-width: 600px; margin: 0 auto; padding: 20px; }"
                </style>
            </head>
            <body>(slot?)</body>
        </html>
    }
}
