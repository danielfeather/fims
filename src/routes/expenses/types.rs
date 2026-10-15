use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Annually,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BusinessDayConvention {
    None,
    Preceding,
    Following,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    GBP,
    EUR,
    USD,
}
