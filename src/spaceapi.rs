/// Defintion of structs for SpaceApi v15
/// Also compatible with v14
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SpaceApi {
    pub api_compatibility: Vec<String>,
    pub space: String,
    pub logo: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    pub contact: Contact,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensors: Option<Sensors>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub address: String,
    pub lon: f64,
    pub lat: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mastodon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ml: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_mail: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastchange: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Icon {
    pub open: String,
    pub closed: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Sensors {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub temperature: Vec<Sensor<f64>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub humidity: Vec<Sensor<f64>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub carbondioxide: Vec<Sensor<u64>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Sensor<T> {
    pub location: String,
    pub unit: String,
    pub value: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastchange: Option<u64>,
}
