use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub pid: Option<u32>,
    pub executable: String,
    #[serde(default = "default_working_dir")]
    pub working_dir: String,
    pub status: String,
    pub started_at: Option<String>,
}

fn default_working_dir() -> String {
    ".".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListProcess {
    #[serde(default)]
    pub processed: Vec<Process>,
}
