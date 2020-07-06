use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Params {
    #[serde(default)]
    pub a: Option<bool>,
    #[serde(default)]
    pub b: Option<bool>,
    #[serde(default)]
    pub c: Option<bool>,
    #[serde(default)]
    pub d: Option<f64>,
    #[serde(default)]
    pub e: Option<i32>,
    #[serde(default)]
    pub f: Option<i32>,
    #[serde(default)]
    pub case: Option<Case>,
}
#[derive(Debug, Serialize)]
pub struct Output {
    pub h: H,
    pub k: f64,
}

#[derive(Debug, Serialize)]
pub enum H {
    M,
    P,
    T,
    E,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Case {
    B,
    C1,
    C2
}

impl Default for H {
    fn default() -> Self {
        H::M
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorMessage {
    pub code: u16,
    pub message: String,
}

