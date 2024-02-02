mod create;
mod edit;
mod list;
mod utils;

use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf};

use clap::{Parser, Subcommand};
use list::ShowMode;
use serde::{Deserialize, Serialize};

const DEFAULT_TIME: &str = "11:59pm";
const CONFIG_FILE_ENV_VAR: &str = "KASK_CONFIG_FILE";

fn main() {
    let args: Args = Args::parse();

    let config_option = utils::get_kask_config_file();
    if let None = config_option {
        return;
    };

    let config = config_option.unwrap();

    let current_list = &config.current_tasks_list;
    let current_list_path = &config.tasks_lists_paths[current_list];
    println!("current list: {}", current_list);
    println!("current list path: {}", current_list_path);
    println!("-------------------------------------------------------------");
    let mut tasks = utils::load_tasks_from_file(current_list_path);

    match args.task_command {
        TaskCommand::Create {
            name,
            description,
            date,
            time,
            tags,
        } => {
            let id = tasks.last().map(|task| task.id + 1).unwrap_or(1);
            let task = create::create_task(name, description, date, time, tags, id);
            if task.is_none() {
                return;
            }
            utils::append_task_to_file(task.unwrap(), current_list_path);
        }
        TaskCommand::List {
            today,
            week,
            month,
            show_mode,
            count,
        } => {
            list::list_tasks(tasks, today, week, month, show_mode, count);
        }
        TaskCommand::Update {
            id,
            name,
            date,
            description,
            time,
            done,
            tags,
        } => {
            if let Err(error) =
                edit::edit_task(&mut tasks, id, name, description, date, time, done, tags)
            {
                println!("Error: {}", error);
                return;
            }
            utils::write_tasks_to_file(current_list_path, tasks);
            println!("Task updated successfully");
        }
        TaskCommand::Delete { id } => {
            let new_tasks: Vec<Task> = tasks.into_iter().filter(|task| task.id != id).collect();
            utils::write_tasks_to_file(current_list_path, new_tasks);
            println!("Task deleted successfully");
        }
        TaskCommand::Complete { id } => {
            let new_tasks: Vec<Task> = tasks
                .into_iter()
                .map(|task| {
                    if task.id == id {
                        let mut new_task = task.clone();
                        new_task.done = true;
                        new_task
                    } else {
                        task.clone()
                    }
                })
                .collect();
            utils::write_tasks_to_file(current_list_path, new_tasks);
            println!("Task completed successfully");
        }
        TaskCommand::Search {
            query,
            start_date,
            end_date,
            tags,
            count,
        } => {
            list::search_tasks(tasks, query, start_date, end_date, tags, count);
        }
        TaskCommand::Config { config_command } => {
            match config_command {
                ConfigCommand::Set { list } => {
                    if !config.tasks_lists_paths.contains_key(&list) {
                        println!("Error: Task list {} does not exist", list);
                        return;
                    }
                    let mut new_config = config.clone();
                    new_config.current_tasks_list = list.clone();
                    if let Err(error) = utils::write_config_to_file(new_config) {
                        println!("Error: {}", error);
                        return;
                    };
                    println!("Current task list set to {}", list);
                }
                ConfigCommand::Add { list, path } => {
                    if config.tasks_lists_paths.contains_key(&list) {
                        println!("Error: Task list {} already exists", list);
                        return;
                    }
                    // check if file exists and if it does not then create it and
                    // notify the user that the file was created
                    if !std::path::Path::new(&path).exists() {
                        if let Err(error) = std::fs::File::create(&path) {
                            println!("Error: {}", error);
                            return;
                        }
                        println!("File {} created successfully", path);
                    }

                    let mut new_config = config.clone();
                    let path = PathBuf::from(path).canonicalize().unwrap();
                    new_config.tasks_lists_paths.insert(
                        list.clone(),
                        path.to_str().unwrap().to_string(),
                    );
                    if let Err(error) = utils::write_config_to_file(new_config) {
                        println!("Error: {}", error);
                        return;
                    };
                    println!("Task list {} added successfully", list);
                }
                ConfigCommand::Remove { list } => {
                    if !config.tasks_lists_paths.contains_key(&list) {
                        println!("Error: Task list {} does not exist", list);
                        return;
                    }
                    let mut new_config = config.clone();
                    new_config.tasks_lists_paths.remove(&list);
                    if let Err(error) = utils::write_config_to_file(new_config) {
                        println!("Error: {}", error);
                        return;
                    };
                    println!("Task list {} removed successfully", list);
                }
                ConfigCommand::Info {} => {
                    println!("Current task list: {}", config.current_tasks_list);
                    println!("Task Lists:");
                    for (list, path) in config.tasks_lists_paths.iter() {
                        println!("\t{}: {}", list, path);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct KaskConfig {
    current_tasks_list: String,
    tasks_lists_paths: HashMap<String, String>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    task_command: TaskCommand,
    #[clap(short, long)]
    /// Path to the task list file to use
    task_file: Option<String>,
}

#[derive(Subcommand, Debug)]
enum TaskCommand {
    /// Create a new task and add it to the current list
    Create {
        name: String,
        date: String,
        #[clap(short = 'm', long)]
        description: Option<String>,
        #[clap(short, long)]
        time: Option<String>,
        #[clap(long)]
        tags: Option<Vec<String>>,
    },
    /// List tasks from the current list. Tasks will be sorted by date and by time
    /// completed tasks will not be shown by default. Use the --show-mode option to
    /// change this behavior
    List {
        /// Show tasks for today
        #[clap(short, long, group = "list_group")]
        today: bool,
        /// Show tasks for the current week
        #[clap(short, long, group = "list_group")]
        week: bool,
        /// Show tasks for the current month
        #[clap(short, long, group = "list_group")]
        month: bool,
        /// Show mode
        #[clap(short, long, value_enum, default_value = "not-done")]
        show_mode: ShowMode,
        /// Number of tasks to display
        #[clap(short, long, default_value = "10")]
        count: u32,
    },
    /// Update a task from the current list by its id
    Update {
        id: u32,
        #[clap(short, long)]
        name: Option<String>,
        #[clap(short, long)]
        date: Option<String>,
        #[clap(short = 'm', long)]
        description: Option<String>,
        #[clap(short, long)]
        time: Option<String>,
        #[clap(long)]
        tags: Option<Vec<String>>,
        #[clap(long)]
        done: Option<bool>,
    },
    /// Delete a task from the current list by its id
    Delete { id: u32 },
    /// Mark a task as complete by its id
    Complete { id: u32 },
    /// Search for tasks in the current list
    Search {
        query: String,
        #[clap(short, long)]
        start_date: Option<String>,
        #[clap(short, long)]
        end_date: Option<String>,
        #[clap(long)]
        tags: Option<Vec<String>>,
        #[clap(short, long, default_value = "10")]
        count: u32,
    },
    /// Configuration commands
    Config {
        #[clap(subcommand)]
        config_command: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Set the current task list
    Set { list: String },
    /// Add a new task list
    Add { list: String, path: String },
    /// Remove a task list
    Remove { list: String },
    /// Dispaly Configuration information
    Info {},
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub date: String,
    pub time: String,
    pub description: String,
    pub done: bool,
    pub tags: Vec<String>,
}

impl FromStr for Task {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(",").collect();
        if parts.len() != 7 {
            return Err(format!(
                "Invalid number of parts in task string: expected 7, found {}",
                parts.len()
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|e| e.to_string())?;
        let name = parts[1].to_string();
        let date = parts[2].to_string();
        let time = parts[3].to_string();
        let description = parts[4].to_string();
        let done = parts[5].trim().parse::<bool>().map_err(|e| e.to_string())?;
        let tags = parts[6]
            .split(";")
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>();

        Ok(Task {
            id,
            name,
            date,
            time,
            description,
            done,
            tags,
        })
    }
}
