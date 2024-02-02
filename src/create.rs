use chrono::NaiveDate;

use crate::{Task, DEFAULT_TIME};


pub fn create_task(
    name: String,
    description: Option<String>,
    date: String,
    time: Option<String>,
    tags: Option<Vec<String>>,
    generated_id: u32,
) -> Option<Task> {
    let date_parsed = NaiveDate::parse_from_str(&date, "%m/%d/%y");
    if date_parsed.is_err() {
        eprintln!("Invalid date format: {} (expected mm/dd/yy)", date);
        return None;
    }

    if let Some(time) = &time {
        let time_parsed = chrono::NaiveTime::parse_from_str(&time, "%I:%M%p");
        if time_parsed.is_err() {
            eprintln!("Invalid time format: {} (expected hh:mm[am|pm])", time);
            return None;
        }
    };

    Some(Task {
        id: generated_id,
        name,
        date,
        time: time.unwrap_or(String::from(DEFAULT_TIME)),
        description: description.unwrap_or(String::from("")),
        done: false,
        tags: tags.unwrap_or(Vec::new()),
    })
}

