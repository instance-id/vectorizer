use std::{path::{PathBuf, Path}, env, io::Write};

use simplelog::*;
use std::fs::File;

use crate::SETTINGS;

pub fn get_config_path(name: &str) -> PathBuf {
    use std::fs;

    let key = "HOME";
    match env::var(key) {
        Ok(val) => debug!("{}: {:?}", key, val),
        Err(e) => error!("couldn't interpret {}: {}", key, e),
    }

    let mut path = PathBuf::new();
    let mut home = env::var(key).expect("Failed to get home dir");
    home.push_str(format!("/.config/{}", name.to_lowercase()).as_str());
    path.push(home);

    fs::create_dir_all(path.clone()).expect("Failed to create config dir");

    if Path::new(&path).exists() {
        let mut settings_path = match path.join("settings.toml").to_str() {
            Some(p) => PathBuf::from(p),
            None => std::path::PathBuf::from("settings"),
        };

        if !Path::new(&settings_path).exists() {
            let settings_file = default_settings(path.clone());
            settings_path = settings_file;
        }

        debug!("{:?}", settings_path);
        let mut settings = SETTINGS.write().unwrap();
        if let Err(err) = settings.merge(config::File::from(settings_path)) {
            warn!("settings merge failed, use default settings, err: {}", err);
        }
    } else {
        warn!("Failed to get config directory or file");
    }

    path
}

pub fn default_settings(settings: PathBuf) -> PathBuf {
    let settings_path = Path::new(&settings).join("settings.toml");

    let mut settings_file = File::create(&settings_path).unwrap();
    let settings_toml = r##"
[indexer]
log_level = "warn"
project_file = true # Create a project settings override file
extensions   = []   # List of file extensions to index
directories  = []   # List of directories to include within the project root
ignored      = []   # List of directories to ignore within the project root

[database]
url          = ""   # URL to the database
collection   = ""   # Name of the collection to create
max_tokens   = 0    # Maximum tokens per fragment when splitting documents
metadata     = ""   # Additional Metadata to add, in json format
"##;

    let toml = settings_toml.replace("{{PATH}}", &settings.to_str().unwrap());
    settings_file.write_all(toml.as_bytes()).unwrap();
    settings_path
}

// --| Default project settings -------
pub fn default_project_settings(settings: PathBuf) -> PathBuf {
    let settings_path = Path::new(&settings).join(".vectorizer");

    let mut settings_file = File::create(&settings_path).unwrap();
    let settings_toml = r##"
[indexer]
extensions   = []   # List of file extensions to index
directories  = []   # List of directories to include within the project root
ignored      = []   # List of directories to ignore within the project root

[database]
url          = ""   # URL to the database
collection   = ""   # Name of the collection to create
max_tokens   = 0    # Maximum tokens per fragment when splitting documents
metadata     = ""   # Additional Metadata to add, in json format
"##;

    settings_file.write_all(settings_toml.as_bytes()).unwrap();
    settings_path
}
