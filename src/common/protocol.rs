use serde::{Deserialize, Serialize};

use crate::model::Process;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Mold { name: String, executable: String },
    Ignite { name: String },
    Quench { name: String },
    Anvils,
    Smelt { name: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Success { message: String },
    Anvils { processes: Vec<Process> },
    Error { message: String },
}
