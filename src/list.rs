use chrono::{Datelike, Local, NaiveDate};
use clap::ValueEnum;

use crate::Task;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ShowMode {
    NotDone, // shows only not done this is the default
    All,     // shows done and not done
    Done,    // shows only done
}

// search_tasks(tasks, query, start_date, end_date, tags, count);
pub fn search_tasks(tasks: Vec<Task>, query: String, start_date: Option<String>, end_date: Option<String>, tags: Option<Vec<String>>, count: u32) {
    // if start date is not specified then use today's date
    let start_date: NaiveDate = if start_date.is_none() {
        let today = Local::now();
        NaiveDate::from_ymd_opt(today.date_naive().year(), today.month(), today.day()).unwrap()
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
    println!(
        "Top {} results",
        std::cmp::min(count, filtered_tasks.len() as u32)
    );
    println!("-----------------");
    println!("{:>3}| {:>30} {:^11}", "ID", "Name", "Date");
    for task in filtered_tasks.iter().take(count as usize) {
        println!("{:>3}| {:>30} {:^11}", task.id, task.name, task.date);
    }
}

pub fn list_tasks(tasks: Vec<Task>, today: bool, week: bool, month: bool, show_mode: ShowMode) {
    let today_value = chrono::Local::now().date_naive();
    let mut tasks_to_show: Vec<Task> = Vec::new();
    if show_mode == ShowMode::All {
        tasks_to_show = tasks;
    } else if show_mode == ShowMode::Done {
        tasks_to_show = tasks.into_iter().filter(|task| task.done).collect();
    } else if show_mode == ShowMode::NotDone {
        tasks_to_show = tasks.into_iter().filter(|task| !task.done).collect();
    }

    if today {
        // filter out tasks that are not today
        tasks_to_show = tasks_to_show
            .into_iter()
            .filter(|task| {
                NaiveDate::parse_from_str(&task.date, "%m/%d/%y").unwrap() == today_value
            })
            .collect();
    } else if week {
        // filter out tasks that are not this week, starting from today and ending the
        // sunday of the same week

        let today_weekday = today_value.weekday().num_days_from_monday();
        let sunday = today_value + chrono::Duration::days(6 - today_weekday as i64);
        tasks_to_show = tasks_to_show
            .into_iter()
            .filter(|task| {
                let task_date = NaiveDate::parse_from_str(&task.date, "%m/%d/%y").unwrap();
                task_date >= today_value && task_date <= sunday
            })
            .collect();
    } else if month {
        let today_month = today_value.month();
        tasks_to_show = tasks_to_show
            .into_iter()
            .filter(|task| {
                let task_date = NaiveDate::parse_from_str(&task.date, "%m/%d/%y").unwrap();
                task_date.month() == today_month
            })
            .collect();
    }
    println!(
        "{:>3}|{:^20} {:^13} {:^7} {:^11}",
        "ID", "Name", "Due Date", "Time", "Done"
    );
    println!("-------------------------------------------------------------");
    for task in tasks_to_show {
        println!(
            "{:>3}|{:^20}|{:^13} {:^7} {:^11}",
            task.id, task.name, task.date, task.time, task.done,
        );
    }
}
