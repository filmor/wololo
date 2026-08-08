use topcoat::{asset::asset, router::layout, view::view, Result};

const PURE_CSS: topcoat::asset::Asset = asset!("assets/pure-min.css");

#[layout("/")]
pub(crate) async fn base(slot: Result) -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8">
                <title>"WoLolo"</title>
                <link rel="stylesheet" type="text/css" href=(PURE_CSS)>
                <style>
                    ".center-column { max-width: 600px; margin: 0 auto; padding: 20px; }"
                </style>
            </head>
            <body>(slot?)</body>
        </html>
    }
}
