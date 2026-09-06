use gpui::{Global, SharedString};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    All,
    Active,
    Done,
}

impl Filter {
    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "全部",
            Filter::Active => "进行中",
            Filter::Done => "已完成",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: u64,
    pub title: SharedString,
    pub done: bool,
}

pub struct AppState {
    pub tasks: Vec<Task>,
    pub filter: Filter,
    pub next_id: u64,
    pub new_task_input: SharedString,
    pub selected_task_id: Option<u64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: vec![
                Task {
                    id: 1,
                    title: "欢迎使用 GPUI 客户端".into(),
                    done: false,
                },
                Task {
                    id: 2,
                    title: "点击左侧筛选任务".into(),
                    done: false,
                },
                Task {
                    id: 3,
                    title: "按 Enter 添加新任务".into(),
                    done: true,
                },
            ],
            filter: Filter::All,
            next_id: 4,
            new_task_input: SharedString::default(),
            selected_task_id: None,
        }
    }

    pub fn filtered_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| match self.filter {
                Filter::All => true,
                Filter::Active => !t.done,
                Filter::Done => t.done,
            })
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.done).count()
    }

    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }

    pub fn add_task(&mut self, title: &str) {
        let title = title.trim();
        if title.is_empty() {
            return;
        }
        self.tasks.push(Task {
            id: self.next_id,
            title: title.trim().to_string().into(),
            done: false,
        });
        self.next_id += 1;
        self.new_task_input = SharedString::default();
    }

    pub fn toggle_task(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.done = !task.done;
        }
    }

    pub fn delete_task(&mut self, id: u64) {
        self.tasks.retain(|t| t.id != id);
        if self.selected_task_id == Some(id) {
            self.selected_task_id = None;
        }
    }

    pub fn clear_done(&mut self) {
        self.tasks.retain(|t| !t.done);
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }
}

impl Global for AppState {}
