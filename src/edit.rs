use crate::Task;

pub fn edit_task(
    tasks: &mut Vec<Task>,
    id: u32,
    name: Option<String>,
    description: Option<String>,
    date: Option<String>,
    time: Option<String>,
    done: Option<bool>,
    tags: Option<Vec<String>>,
) -> Result<(), String>{
    let task = tasks.iter_mut().find(|task| task.id == id).ok_or("Task not found")?;

    if let Some(name) = name {
        task.name = name;
    }
    if let Some(date) = date {
        task.date = date;
    }
    if let Some(description) = description {
        task.description = description;
    }
    if let Some(time) = time {
        task.time = time;
    }
    if let Some(tags) = tags {
        task.tags = tags;
    }
    if let Some(done) = done {
        task.done = done;
    }

    Ok(())
}

pub fn complete_task(tasks: &mut Vec<Task>, id: u32) {
    for task in tasks {
        if task.id == id {
            task.done = true;
            break;
        }
    }
}

pub fn delete_task(tasks: &mut Vec<Task>, id: u32) {
    tasks.retain(|task| task.id != id);
}
