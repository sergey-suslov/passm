use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

const DEFAULT_BASE_PATH: &str = "./.passm";
const DEFAULT_NAMESPACE_NAME: &str = "default";
const DEFAULT_PRIVATE_KEY_NAME: &str = ".private_1";
const MAIN_CONFIG_NAME: &str = ".config.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct NamespaceConfig {
    pub private_key_path: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    configurations: Vec<String>,
    default: String,
}

impl Configuration {
    fn get_namespace_config_path(base_path: PathBuf, name: &str) -> PathBuf {
        base_path.join(format!(".{}.config.toml", name))
    }
    fn get_main_config_path(base_path: PathBuf) -> PathBuf {
        base_path.join(MAIN_CONFIG_NAME)
    }
    pub fn init_new(
        path: String,
        default_namespace_name: Option<String>,
    ) -> Result<NamespaceConfig> {
        /*
         *  If dir exists, check that MAIN_CONFIGURATION_FILE_NAME does not exists.
         *  If dir does not exists, create it, create MAIN_CONFIGURATION_FILE_NAME file.
         * */
        let base_path = Path::new(&path);
        let default_namespace = default_namespace_name.unwrap_or_else(|| String::from("default"));
        let main_config_path = Configuration::get_main_config_path(base_path.to_path_buf());

        fs::create_dir(base_path).unwrap();

        let namespace_config = Configuration::init_new_or_read_existing_namespace_config(
            base_path.to_path_buf(),
            &default_namespace,
        )?;

        let main_config = Configuration {
            configurations: vec![default_namespace],
            default: namespace_config.name.clone(),
        };
        fs::write(main_config_path, toml::to_string(&main_config).unwrap()).unwrap();
        Ok(namespace_config)
    }
    pub fn init_new_or_read_existing_namespace_config(
        base_path: PathBuf,
        name: &str,
    ) -> Result<NamespaceConfig> {
        let namespace_config_path =
            Configuration::get_namespace_config_path(base_path.clone(), name);
        match fs::metadata(&namespace_config_path) {
            Ok(_) => {
                let existing_config: NamespaceConfig =
                    toml::from_str(&fs::read_to_string(namespace_config_path).unwrap()).unwrap();
                Ok(existing_config)
            }
            _ => {
                let namespace_config = NamespaceConfig {
                    private_key_path: base_path
                        .join(DEFAULT_PRIVATE_KEY_NAME)
                        .to_str()
                        .unwrap()
                        .to_string(),
                    name: DEFAULT_NAMESPACE_NAME.to_string(),
                };
                fs::write(
                    namespace_config_path,
                    toml::to_string(&namespace_config).unwrap(),
                )
                .unwrap();
                Ok(namespace_config)
            }
        }
    }
    pub fn init_from_path(path: String) -> Result<NamespaceConfig> {
        let base_path = Path::new(&path);
        let config_path = Configuration::get_main_config_path(base_path.to_path_buf());
        match fs::metadata(&config_path) {
            Ok(_) => {
                println!("Config path exists");
            }
            _ => return Err(anyhow!("No main configuration file found")),
        }
        let existing_config: Configuration =
            toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();

        let namespace_config_path = Configuration::get_namespace_config_path(
            base_path.to_path_buf(),
            &existing_config.default,
        );
        let existing_namespace_config: NamespaceConfig =
            toml::from_str(&fs::read_to_string(namespace_config_path).unwrap()).unwrap();
        Ok(existing_namespace_config)
    }
    pub fn init() -> Result<NamespaceConfig> {
        let base_path = Path::new(DEFAULT_BASE_PATH);
        let config = match fs::metadata(base_path) {
            Ok(_) => Configuration::init_from_path(DEFAULT_BASE_PATH.to_string()),
            Err(_) => Configuration::init_new(DEFAULT_BASE_PATH.to_string(), None),
        };
        if let Err(err) = config {
            println!("Error: {}", err);
            process::exit(1)
        }
        Ok(config.unwrap())
    }
}
