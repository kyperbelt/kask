use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};
use std::io::Write;
use std::str::FromStr;

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


pub fn get_cask_config_file() -> Option<KaskConfig> {
    // search for the environment variable
    let env_config_file = env::var(CONFIG_FILE_ENV_VAR);

    // if env variable found then load it and return it
    if let Ok(path_from_var) = env_config_file {
        if let Ok(file_string) = fs::read_to_string(&path_from_var) {
            let config: KaskConfig = serde_json::from_str(&file_string).unwrap();
            println!("Using config from environment at {}", &path_from_var);
            return Some(config);
        }
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
    let mut tasks_lists_paths: HashMap<String, String> = HashMap::new();
    tasks_lists_paths.insert("default_tasks".to_string(), "default_tasks.csv".to_string());
    let config = KaskConfig {
        current_tasks_list: "default_tasks".to_string(),
        tasks_lists_paths,
    };

    let config_json_string = serde_json::to_string(&config).unwrap();
    if let Err(error) = fs::write("kask.config", config_json_string) {
        println!("Error creating local kask.config file: {}", error);
        return None;
    };

    println!("Created new local kask.config file.");

    return Some(config);
}

