use chrono::{Datelike, NaiveDate};
use clap::ValueEnum;

use crate::Task;



#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ShowMode {
    NotDone, // shows only not done this is the default
    All,     // shows done and not done
    Done,    // shows only done
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
