use std::{fs, io::Write};

use crate::{error::Result, model::ListProcess};

const PROCESS_JSON: &str = "./data/process.json";

pub fn get_process() -> Result<ListProcess> {
    match fs::read_to_string(PROCESS_JSON) {
        Ok(json) => Ok(serde_json::from_str(&json)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ListProcess {
            processed: Vec::new(),
        }),
        Err(error) => Err(error.into()),
    }
}

pub fn save(list: &ListProcess) -> Result<()> {
    let json_string = serde_json::to_string_pretty(list)?;
    fs::create_dir_all("./data")?;
    let mut file = fs::File::create(PROCESS_JSON)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}
