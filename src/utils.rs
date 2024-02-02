use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};

use crate::{KaskConfig, Task, CONFIG_FILE_ENV_VAR};

pub fn parse_tags(tags: Option<Vec<String>>) -> String {
    match tags {
        Some(tags) => tags.join("; "),
        None => String::from(""),
    }
}

pub fn write_tasks_to_file(filename: &str, tasks: Vec<Task>) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(filename)
        .unwrap();

    for task in tasks {
        let store_string: String = format!(
            "{}, {}, {}, {}, {}, {}, {}",
            task.id,
            task.name,
            task.date,
            task.time,
            task.description,
            task.done.to_string(),
            task.tags.join("; ")
        );

        if let Err(e) = writeln!(file, "{}", store_string) {
            eprintln!("Error writing to file: {}", e);
        };
    }
}

pub fn append_task_to_file(task: Task, filename: &str) {
    let store_string: String = format!(
        "{}, {}, {}, {}, {}, {}, {}",
        task.id,
        task.name,
        task.date,
        task.time,
        task.description,
        task.done.to_string(),
        task.tags.join("; ")
    );

    // if the file does not exist create it
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(filename)
        .unwrap();

    if let Err(e) = writeln!(file, "{}", store_string) {
        eprintln!("Error writing to file: {}", e);
    };
}

pub fn load_tasks_from_file(filename: &str) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();

    // if the file does not exist create it

    if !PathBuf::from(filename).exists() {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .unwrap();
        println!("new tasklist file created at {}", filename);
        return tasks;
    }

    let file = fs::read_to_string(filename);
    if file.is_err() {
        eprintln!("Error reading file: {}", file.unwrap_err());
        return tasks;
    }

    let file = file.unwrap();
    for line in file.lines() {
        let task = Task::from_str(line);
        if task.is_err() {
            eprintln!("Error parsing task: {}", task.unwrap_err());
            continue;
        }
        tasks.push(task.unwrap());
    }

    tasks
}

fn new_kask_config() -> KaskConfig {
    let mut tasks_lists_paths: HashMap<String, String> = HashMap::new();
    let path_prefix = Path::new(".").canonicalize().unwrap();
    let filepath = path_prefix.join("default_tasks.csv");
    tasks_lists_paths.insert(
        "default_tasks".to_string(),
        filepath.to_str().unwrap().to_string(),
    );
    KaskConfig {
        current_tasks_list: "default_tasks".to_string(),
        tasks_lists_paths,
    }
}

pub fn get_kask_config_file() -> Option<KaskConfig> {
    // search for the environment variable
    let env_config_file = env::var(CONFIG_FILE_ENV_VAR);

    // if env variable found then load it and return it
    if let Ok(path_from_var) = env_config_file {
        // if the file exists then load it
        if let Ok(file_string) = fs::read_to_string(&path_from_var) {
            let config: KaskConfig = serde_json::from_str(&file_string).unwrap();
            println!("Using config file at {}", &path_from_var);
            return Some(config);
        };
        let config = new_kask_config();
        let config_json_string = serde_json::to_string(&config).unwrap();
        if let Err(error) = fs::write(&path_from_var, config_json_string) {
            println!("Error creating config file: {}", error);
            return None;
        };

        return Some(config);
    };

    // if env variable not found then search for a file in a folder ~/.config/kask/kask.config

    let file_path: String = format!("{}/.config/kask/kask.config", env::var("HOME").unwrap());
    if let Ok(file_string) = fs::read_to_string(&file_path) {
        let config: KaskConfig = serde_json::from_str(&file_string).unwrap();
        println!("Using config file at {}", &file_path);
        return Some(config);
    };
    // if that file is not found then search for a local file named kask.config
    if let Ok(file_string) = fs::read_to_string("kask.config") {
        let config: KaskConfig = serde_json::from_str(&file_string).unwrap();
        println!(
            "Using local config at {}/kask.config",
            env::current_dir()
                .unwrap_or(PathBuf::from("."))
                .as_path()
                .to_str()
                .unwrap()
        );
        return Some(config);
    };

    // if thats not found then create a local file named kask.config and create it.
    let config = new_kask_config();
    let config_json_string = serde_json::to_string(&config).unwrap();
    if let Err(error) = fs::write("kask.config", config_json_string) {
        println!("Error creating local kask.config file: {}", error);
        return None;
    };

    println!("Created new local kask.config file.");

    return Some(config);
}

pub fn write_config_to_file(config: KaskConfig) -> Result<(), std::io::Error> {
    let config_json_string = serde_json::to_string(&config).unwrap();
    fs::write(get_config_file_path().unwrap(), config_json_string)
}

fn get_config_file_path() -> Option<String> {
    // search for the environment variable
    let env_config_file = env::var(CONFIG_FILE_ENV_VAR);

    // if env variable found then load it and return it
    if let Ok(path_from_var) = env_config_file {
        return Some(path_from_var);
    };

    // if env variable not found then search for a file in a folder ~/.config/kask/kask.config
    let file_path: String = format!("{}/.config/kask/kask.config", env::var("HOME").unwrap());
    if fs::metadata(&file_path).is_ok() {
        return Some(file_path);
    };

    // if that file is not found then search for a local file named kask.config
    if fs::metadata("kask.config").is_ok() {
        return Some("kask.config".to_string());
    };

    return None;
}
