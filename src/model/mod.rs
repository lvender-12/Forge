use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub pid: Option<u32>,
    pub executable: String,
    pub status: String,
    pub started_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListProcess {
    #[serde(default)]
    pub processed: Vec<Process>,
}
