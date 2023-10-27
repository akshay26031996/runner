use dashmap::DashMap;
use serde::Deserialize;

pub type State = DashMap<String, AppState>;

#[derive(Debug, PartialEq, Clone)]
pub enum AppState {
    PENDING,
    STARTED,
    RUNNING,
    ERROR,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub app: String,
    pub start_command: String,
    pub health_check_url: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum StepConfig {
    Serial(AppConfig),
    Parallel(Vec<AppConfig>),
}

#[derive(Deserialize, Debug)]
pub struct Process {
    pub steps: Vec<StepConfig>,
}
