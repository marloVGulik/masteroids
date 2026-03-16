

pub struct Task<T> {
    name: String,
    start_time: std::time::Instant,
    wait_time: std::time::Duration,
    repeat: bool,
    has_ran: bool,
    action: T,
}
pub struct Scheduler<T> {
    tasks: Vec<Task<T>>,
}

impl<T> Scheduler<T> {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn schedule(&mut self, name: String, wait_time: std::time::Duration, repeat: bool, action: T) {
        self.tasks.push(Task {
            name,
            start_time: std::time::Instant::now(),
            wait_time,
            repeat,
            has_ran: false,
            action: action,
        });
    }
    pub fn unschedule(&mut self, name: &str) {
        self.tasks.retain(|task| task.name != name);
    }

    pub fn update(&mut self, mut handler: impl FnMut(&mut T)) {
        let now = std::time::Instant::now();
        for task in &mut self.tasks {
            if now.duration_since(task.start_time) >= task.wait_time {
                handler(&mut task.action);
                if task.repeat {
                    task.start_time = now;
                }
                task.has_ran = true;
            }
        }
        self.tasks.retain(|task| !(task.has_ran && !task.repeat));
    }
}