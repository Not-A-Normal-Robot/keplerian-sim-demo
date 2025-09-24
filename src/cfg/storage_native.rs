use directories::ProjectDirs;
use std::{
    env,
    error::Error,
    ffi::OsString,
    fmt::Display,
    fs::OpenOptions,
    io::{self, Read, Write},
    path::PathBuf,
    process::Command,
    sync::LazyLock,
};

use super::super::HALT_FLAG;

use serde::{Deserialize, Serialize};

static PROJECT_DIRS: LazyLock<Option<ProjectDirs>> =
    LazyLock::new(|| ProjectDirs::from("io.github", "Not-A-Normal-Robot", "keplerian_sim_demo"));
static CONFIG_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    PROJECT_DIRS
        .as_ref()
        .map(|dir| dir.config_dir().to_path_buf())
});
static CONFIG_PATH: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| CONFIG_DIR.as_ref().map(|dir| dir.join("config.toml")));
static TEMP_CONFIG_PATH: LazyLock<Option<PathBuf>> =
    LazyLock::new(|| CONFIG_DIR.as_ref().map(|dir| dir.join("config.toml.tmp")));

fn get_table(cfg_path: &PathBuf) -> toml::value::Table {
    let Ok(mut file) = OpenOptions::new().read(true).open(cfg_path) else {
        return toml::value::Table::new();
    };

    let mut string = String::new();
    let Ok(_) = file.read_to_string(&mut string) else {
        return toml::value::Table::new();
    };

    let Ok(toml_value) = toml::from_str(&string) else {
        return toml::value::Table::new();
    };

    match toml_value {
        toml::Value::Table(t) => t,
        _ => toml::value::Table::new(),
    }
}

pub(super) fn save<T: Serialize>(key: &str, value: T) -> Result<(), SaveError> {
    let cfg_dir = CONFIG_DIR.as_ref().ok_or(SaveError::NoSaveDirectory)?;
    let cfg_path = CONFIG_PATH.as_ref().ok_or(SaveError::NoSaveDirectory)?;
    let tmp_path = TEMP_CONFIG_PATH
        .as_ref()
        .ok_or(SaveError::NoSaveDirectory)?;

    let value = toml::value::Value::try_from(value).map_err(|e| SaveError::SerializeValue(e))?;

    std::fs::create_dir_all(cfg_dir).map_err(|e| SaveError::CreateConfigDir(e))?;

    let mut table = get_table(cfg_path);

    let mut tmp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(tmp_path)
        .map_err(|e| SaveError::OpenTmpFile(e))?;

    table.insert(key.to_string(), value);
    let table_string = toml::to_string(&table).map_err(|e| SaveError::StringifyTable(e))?;

    tmp_file
        .write_all(table_string.as_bytes())
        .map_err(|e| SaveError::Write(e))?;
    tmp_file.flush().map_err(|e| SaveError::Write(e))?;
    drop(tmp_file);

    std::fs::rename(tmp_path, cfg_path).map_err(|e| SaveError::Rename(e))?;

    Ok(())
}
pub(super) fn load<T: for<'d> Deserialize<'d>>(key: &str) -> Result<T, LoadError> {
    let file = CONFIG_PATH.as_ref().ok_or(LoadError::NoSaveDirectory)?;
    let mut file = OpenOptions::new()
        .read(true)
        .open(file)
        .map_err(|e| LoadError::OpenFile(e))?;

    let mut string = String::new();
    file.read_to_string(&mut string)
        .map_err(|e| LoadError::ReadFile(e))?;
    drop(file);

    let mut table: toml::value::Table =
        toml::from_str(&string).map_err(|e| LoadError::DeserializeFile(e))?;

    let value = table.remove(key).ok_or(LoadError::NotFoundInTable)?;
    value.try_into().map_err(|e| LoadError::DeserializeValue(e))
}

pub(crate) fn reset() -> Result<(), ResetError> {
    let Some(file) = CONFIG_PATH.as_ref() else {
        return Ok(());
    };
    match std::fs::remove_file(file) {
        Ok(_) => (),
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => return Err(ResetError::DeleteConfig(e)),
    };

    let exe = env::current_exe().map_err(|e| ResetError::GetCurrentExe(e))?;

    Command::new(exe)
        .args(env::args_os())
        .envs(
            env::vars_os()
                .skip(1)
                .collect::<Box<[(OsString, OsString)]>>(),
        )
        .spawn()
        .map_err(|e| ResetError::LaunchError(e))?;

    unsafe {
        HALT_FLAG = true;
    }

    Ok(())
}

#[derive(Debug)]
pub(crate) enum SaveError {
    NoSaveDirectory,
    SerializeValue(toml::ser::Error),
    CreateConfigDir(io::Error),
    OpenTmpFile(io::Error),
    StringifyTable(toml::ser::Error),
    Write(io::Error),
    Rename(io::Error),
}

impl Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::NoSaveDirectory => write!(f, "No reasonable save directory was found"),
            SaveError::SerializeValue(error) => write!(f, "SerializeValue: {error}"),
            SaveError::CreateConfigDir(error) => write!(f, "CreateConfigDir: {error}"),
            SaveError::OpenTmpFile(error) => write!(f, "OpenTmpFile: {error}"),
            SaveError::StringifyTable(error) => write!(f, "StringifyTable: {error}"),
            SaveError::Write(error) => write!(f, "Write: {error}"),
            SaveError::Rename(error) => write!(f, "Rename: {error}"),
        }
    }
}

impl Error for SaveError {}

#[derive(Debug)]
pub(crate) enum LoadError {
    NoSaveDirectory,
    OpenFile(io::Error),
    ReadFile(io::Error),
    DeserializeFile(toml::de::Error),
    NotFoundInTable,
    DeserializeValue(toml::de::Error),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::NoSaveDirectory => write!(f, "No reasonable save directory was found"),
            LoadError::OpenFile(error) => write!(f, "OpenFile: {error}"),
            LoadError::ReadFile(error) => write!(f, "ReadFile: {error}"),
            LoadError::DeserializeFile(error) => write!(f, "DeserializeFile: {error}"),
            LoadError::NotFoundInTable => write!(f, "Key not found in table"),
            LoadError::DeserializeValue(error) => write!(f, "DeserializeValue: {error}"),
        }
    }
}

impl Error for LoadError {}

#[derive(Debug)]
pub(crate) enum ResetError {
    DeleteConfig(io::Error),
    GetCurrentExe(io::Error),
    LaunchError(io::Error),
}

impl Display for ResetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResetError::DeleteConfig(error) => write!(f, "DeleteConfig: {error}"),
            ResetError::GetCurrentExe(error) => write!(f, "GetCurrentExe: {error}"),
            ResetError::LaunchError(error) => write!(f, "LaunchError: {error}"),
        }
    }
}
impl Error for ResetError {}
