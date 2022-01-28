
use std::path::PathBuf;

use app_dirs::{app_dir, AppDataType, AppDirsError, AppInfo};



const APP_INFO: AppInfo = AppInfo { name: "eitaro", author: "anekos" };


pub fn get_dictionary_path(name: Option<&str>) -> Result<PathBuf, AppDirsError> {
    let mut result = app_dir(AppDataType::UserCache, &APP_INFO, "dictionary")?;
    if let Some(name) = name {
        result.push(format!("{}.sqlite", name));
    } else {
        result.push("db.sqlite");
    }
    Ok(result)
}

pub fn get_history_path() -> Result<PathBuf, AppDirsError> {
    let mut path = app_dir(AppDataType::UserCache, &APP_INFO, "history")?;
    path.push("history.txt");
    Ok(path)
}
