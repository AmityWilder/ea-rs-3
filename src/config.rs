use crate::{input::Bindings, theme::Theme};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    #[serde(rename = "input")]
    pub binds: Bindings,
}
