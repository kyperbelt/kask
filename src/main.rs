use std::fs;
use std::io::Write;

use chrono::NaiveDate;
use clap::{Parser, Subcommand};


const DEFAULT_TIME: &str = "11:59pm";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    task_command: TaskCommand,
}

#[derive(Subcommand, Debug)]
enum TaskCommand {
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
}

fn main() {
    let args: Args = Args::parse();

    match args.task_command {
        TaskCommand::Create {
            name,
            description,
            date,
            time,
            tags,
        } => {
            let date_parsed = NaiveDate::parse_from_str(&date, "%m/%d/%y");
            if date_parsed.is_err() {
                eprintln!("Invalid date format: {} (expected mm/dd/yy)", date);
                return;
            }

            if let Some(time) = &time {
                let time_parsed = chrono::NaiveTime::parse_from_str(&time, "%I:%M%p");
                if time_parsed.is_err() {
                    eprintln!("Invalid time format: {} (expected hh:mm[am|pm])", time);
                    return;
                }
            }

            // header = "id, name, date, time, description, done, tags"
            // let store_string: String = format!(
            //     "{}, {}, {}, {}, {}, {}, {}",
            //     0.to_string(),
            //     name,
            //     date,
            //     time.unwrap_or(String::from(DEFAULT_TIME)),
            //     description.unwrap_or(
            //         String::from(""),
            //     ),
            //     false.to_string(),
            //     parse_tags(tags)
            // );

            // println!(
            //     "New task created ({}) due {} at {}\n\tdescription:\n\t\t{}",
            //     name,
            //     date,
            //     time.unwrap_or(String::from(DEFAULT_TIME)),
            //     description.unwrap_or(String::from(""))
            // );
            // println!("{}", store_string);
            append_task_to_file(Task {
                id: 0,
                name,
                date,
                time: time.unwrap_or(String::from(DEFAULT_TIME)),
                description: description.unwrap_or(String::from("")),
                done: false,
                tags: tags.unwrap_or(Vec::new()),
            });
        }
    }
}

struct Task {
    id: u32,
    name: String,
    date: String,
    time: String,
    description: String,
    done: bool,
    tags: Vec<String>,
}

fn append_task_to_file(task: Task) {
    let store_string: String = format!(
        "{}, {}, {}, {}, {}, {}, {}",
        0.to_string(),
        task.name,
        task.date,
        task.time,
        task.description,
        task.done.to_string(),
        task.tags.join("; ")
    );
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("tasks.csv")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", store_string){
        eprintln!("Error writing to file: {}", e);
    }
}

fn parse_tags(tags: Option<Vec<String>>) -> String {
    match tags {
        Some(tags) => tags.join("; "),
        None => String::from(""),
    }
}

