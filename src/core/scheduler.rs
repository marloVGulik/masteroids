

struct Task {
    name: String,
    start_time: std::time::Instant,
    wait_time: std::time::Duration,
    repeat: bool,
    action: Box<dyn FnMut()>,
}
struct Scheduler {
    tasks: Vec<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn schedule(&mut self, name: String, wait_time: std::time::Duration, repeat: bool, action: impl FnMut() + 'static) {
        self.tasks.push(Task {
            name,
            start_time: std::time::Instant::now(),
            wait_time,
            repeat,
            action: Box::new(action),
        });
    }
    pub fn unschedule(&mut self, name: &str) {
        self.tasks.retain(|task| task.name != name);
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        for task in &mut self.tasks {
            if now.duration_since(task.start_time) >= task.wait_time {
                (task.action)();
                if task.repeat {
                    task.start_time = now;
                }
            }
        }
        self.tasks.retain(|task| task.repeat || now.duration_since(task.start_time) < task.wait_time);
    }
}