use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item};

pub const CONFIG: Item<State> = Item::new("config");
pub const PRISMFORGE : Item<String> = Item::new("config_prism");
pub const DENOM : Item<String> = Item::new("config_denom");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: String,
    pub denom :String
}

