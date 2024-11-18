use maud::{html, Markup, Render};

pub struct Base;

impl Render for Base {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" type="text/css" href="/assets/pure-min.css";
            style {
                r#"
                .center-column {
                    max-width: 600px; /* Adjust the max-width as needed */
                    margin: 0 auto; /* Center horizontally */
                    padding: 20px;
                }
                "#
            }
        }
    }
}
