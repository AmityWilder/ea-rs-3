use serde_derive::{Deserialize, Serialize};
use {input::Bindings, theme::Theme};

pub mod input;
pub mod theme;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    #[serde(rename = "input")]
    pub binds: Bindings,
}
