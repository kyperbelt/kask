mod create;
mod edit;
mod list;
mod utils;

use std::collections::HashMap;
use std::str::FromStr;

use chrono::{Datelike, Local, NaiveDate};
use clap::{Parser, Subcommand};
use list::ShowMode;
use serde::{Deserialize, Serialize};

const DEFAULT_TIME: &str = "11:59pm";
const CONFIG_FILE_ENV_VAR: &str = "KASK_CONFIG_FILE";

fn main() {
    let config_option = utils::get_cask_config_file();
    if let None = config_option {
        return;
    };

    let args: Args = Args::parse();

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
        } => {
            list::list_tasks(tasks, today, week, month, show_mode);
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
            // if start date is not specified then use today's date
            let start_date: NaiveDate = if start_date.is_none() {
                let today = Local::now();
                NaiveDate::from_ymd_opt(today.date_naive().year(), today.month(), today.day())
                    .unwrap()
            } else {
                NaiveDate::parse_from_str(&start_date.unwrap(), "%m/%d/%y").unwrap()
            };

            // if end date is not specified then search for task from start date until end of time.
            let end_date: NaiveDate = if end_date.is_none() {
                NaiveDate::from_ymd_opt(9999, 12, 31).unwrap()
            } else {
                NaiveDate::parse_from_str(&end_date.unwrap(), "%m/%d/%y").unwrap()
            };

            let mut filtered_tasks = tasks
                .into_iter()
                .filter(|task| {
                    let task_date = NaiveDate::parse_from_str(&task.date, "%m/%d/%y").unwrap();
                    task_date >= start_date && task_date <= end_date
                })
                .collect::<Vec<Task>>();

            filtered_tasks.sort_by(|a, b| {
                let a_title = &a.name;
                let b_title = &b.name;
                let a_distance = strsim::levenshtein(a_title, &query);
                let b_distance = strsim::levenshtein(b_title, &query);
                a_distance.cmp(&b_distance)
            });

            // only print out the top ten results
            println!("Searching for tasks with query: {}", query);
            println!("Start Date: {}", start_date.format("%m/%d/%y"));
            println!("End Date: {}", end_date.format("%m/%d/%y"));
            println!("Top {} results", std::cmp::min(count, filtered_tasks.len() as u32));
            println!("-----------------");
            println!("{:>3}| {:>30} {:^11}", "ID", "Name", "Date");
            for task in filtered_tasks.iter().take(count as usize) {
                println!("{:>3}| {:>30} {:^11}", task.id, task.name, task.date);
            }


        }
        TaskCommand::Config { config_command } => {}
    }
}

#[derive(Serialize, Deserialize)]
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
    /// List tasks from the current list
    List {
        #[clap(short, long, group = "list_group")]
        today: bool,
        #[clap(short, long, group = "list_group")]
        week: bool,
        #[clap(short, long, group = "list_group")]
        month: bool,
        #[clap(short, long, value_enum, default_value = "not-done")]
        show_mode: ShowMode,
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
        count: u32
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
