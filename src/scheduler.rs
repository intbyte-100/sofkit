use std::{
    cell::{Cell, RefCell},
    panic::Location,
    rc::Rc,
};

use gtk::glib;

thread_local! {
    static SCHEDULER: Rc<Scheduler> = Rc::new(Scheduler::new());
}

pub struct Scheduler {
    notify_pass: Cell<u64>,
    frame: Cell<u64>,
    task_queue: RefCell<Vec<Task>>,
}

struct Task {
    task: Option<Box<dyn FnOnce()>>,
    location: &'static Location<'static>,
}

impl Task {
    fn new(location: &'static Location<'static>, task: Box<dyn FnOnce()>) -> Self {
        Self {
            task: Some(task),
            location,
        }
    }

    fn run(&mut self) {
        if let Some(task) = std::mem::take(&mut self.task) {
            task();
        }
    }
}

impl Scheduler {
    fn new() -> Self {
        Self {
            // 0 is invalid for batching logic, first notify pass starts from 1
            notify_pass: Cell::new(1),
            frame: Cell::new(0),
            task_queue: RefCell::new(Vec::new()),
        }
    }

    fn run_traced_tasks(&self, tasks: Vec<Task>) {
        let mut tree = TracerTree::default();
        let mut local_notify_pass = 0;
        tree.insert(0, tasks);

        loop {
            let elements = tree.elements_on_next_level();

            if elements.is_empty() {
                break;
            }

            self.notify_pass.set(self.notify_pass.get() + 1);
            local_notify_pass += 1;

            if local_notify_pass == 25 {
                let trace = tree.build_trace(elements[0]);

                panic!(
                    "Cycle detected in reactive system:\n{}",
                    trace
                        .iter()
                        .map(|loc| format!("  at {}:{}:{}", loc.file(), loc.line(), loc.column()))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }

            for id in elements {
                if let Some(node) = tree.get(id) {
                    node.task.run();
                    tree.insert(id, std::mem::take(&mut self.task_queue.borrow_mut()));
                }
            }
        }
    }

    fn run_tasks(&self) {
        let mut tasks: Vec<_> = std::mem::take(&mut self.task_queue.borrow_mut());
        let mut local_notify_pass = 0;

        loop {
            self.notify_pass.set(self.notify_pass.get() + 1);
            local_notify_pass += 1;

            for mut task in tasks {
                task.run();
            }

            if self.task_queue.borrow().is_empty() {
                break;
            }

            tasks = std::mem::take(&mut self.task_queue.borrow_mut());

            if local_notify_pass > 100 {
                self.run_traced_tasks(tasks);
                break;
            }
        }
    }

    #[track_caller]
    pub fn schedule(self: Rc<Self>, func: impl FnOnce() + 'static) {
        self.task_queue
            .borrow_mut()
            .push(Task::new(std::panic::Location::caller(), Box::new(func)));

        let current_frame = current_reactive_frame();

        if self.frame.get() < current_frame {
            self.frame.set(current_frame);

            glib::idle_add_local_once(move || {
                self.run_tasks();
            });
        }
    }

    pub fn get() -> Rc<Self> {
        SCHEDULER.with(|s| s.clone())
    }

    pub fn notify_pass(&self) -> u64 {
        self.notify_pass.get()
    }
}

struct ReactiveFrame {
    frame: Cell<u64>,
    is_updated: Cell<bool>,
}

impl ReactiveFrame {
    fn new() -> Self {
        Self {
            frame: Cell::new(0),
            is_updated: Cell::new(false),
        }
    }
}

thread_local! {
    static REACTIVE_FRAME: ReactiveFrame = ReactiveFrame::new();
}

pub(crate) fn current_reactive_frame() -> u64 {
    REACTIVE_FRAME.with(|it| {
        if !it.is_updated.get() {
            it.frame.set(it.frame.get() + 1);
            it.is_updated.set(true);

            glib::idle_add_local_once(move || {
                REACTIVE_FRAME.with(|it| {
                    it.is_updated.set(false);
                });
            });
        }

        it.frame.get()
    })
}

#[derive(Default)]
struct TracerTree {
    nodes: Vec<TracerTreeNode>,
    next_level: Vec<usize>,
}

impl TracerTree {
    fn insert(&mut self, parent: usize, tasks: Vec<Task>) {
        if parent != 0 {
            let mut node_id = self.nodes.len();
            let node = &mut self.nodes[parent - 1];
            let level = node.level + 1;

            let nodes: Vec<_> = tasks
                .into_iter()
                .map(|it| {
                    node_id += 1;
                    self.next_level.push(node_id);

                    TracerTreeNode {
                        task: it,
                        parent: parent,
                        level: level,
                    }
                })
                .collect();

            self.nodes.extend(nodes.into_iter());
        } else {
            let mut node_id = self.nodes.len();

            let nodes = tasks.into_iter().map(|it| {
                node_id += 1;
                self.next_level.push(node_id);

                TracerTreeNode {
                    task: it,
                    parent: 0,
                    level: 0,
                }
            });

            self.nodes.extend(nodes);
        }
    }

    fn elements_on_next_level(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.next_level)
    }

    fn get(&mut self, id: usize) -> Option<&mut TracerTreeNode> {
        self.nodes.get_mut(id - 1)
    }

    fn build_trace(&mut self, from: usize) -> Vec<&'static std::panic::Location<'static>> {
        let mut node_id = from;
        let mut trace = Vec::new();

        loop {
            if let Some(node) = self.get(node_id) {
                trace.push(node.task.location);
                node_id = node.parent;
            } else {
                break;
            }

            if node_id == 0 {
                break;
            }
        }

        trace
    }
}

struct TracerTreeNode {
    task: Task,
    parent: usize,
    level: usize,
}
