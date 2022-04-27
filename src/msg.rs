use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Decimal, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub anchor_portion : Decimal,
    pub luna_portion: Decimal,
    pub anchor_address : String,
    pub token_address : String,
    pub denom : String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
 Deposit{},
 Withdraw{amount:Uint128},
 SendToWallet{amount:Uint128},
 SetOwner{address:String},
 ChangePortion {anchor_portion:Decimal,luna_portion:Decimal},
 //SetPrismAddress{address:String}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns a human-readable representation of the arbiter.
    GetStateInfo {},
    GetEpochState {},
    GetAustBalance{}
}
