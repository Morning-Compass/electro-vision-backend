use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    HelpNeeded,
    Todo,
    InProgress,
    Completed,
    Canceled,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Importance {
    Low,
    Medium,
    High,
}
