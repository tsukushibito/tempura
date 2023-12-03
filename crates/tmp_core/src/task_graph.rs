use crate::thread_pool::ThreadPool;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};

/// `TaskCompletion` is a structure to track the completion of a set of tasks.
/// It uses a `Mutex` to safely update the count of remaining tasks across threads
/// and a `Condvar` to provide a way for threads to wait for all tasks to complete.
struct TaskCompletion {
    count: Mutex<usize>,
    condvar: Condvar,
}

impl TaskCompletion {
    /// Creates a new `TaskCompletion` instance with the specified number of tasks.
    ///
    /// # Arguments
    ///
    /// * `count` - The total number of tasks that need to be completed.
    fn new(count: usize) -> Self {
        TaskCompletion {
            count: Mutex::new(count),
            condvar: Condvar::new(),
        }
    }

    /// Marks one task as completed. If all tasks are completed, it notifies all waiting threads.
    ///
    /// This method locks the `Mutex` and safely decrements the task count.
    /// If the count reaches zero, it triggers the `Condvar` to wake up all waiting threads.
    fn task_completed(&self) {
        let mut count = match self.count.lock() {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Failed to lock: {:?}", e);
                return;
            }
        };
        *count -= 1;
        if *count == 0 {
            self.condvar.notify_all();
        }
    }

    /// Waits for all tasks to be completed.
    ///
    /// This method locks the `Mutex` and enters a loop, waiting on the `Condvar` until the task count is zero.
    /// If the `Mutex` lock or `Condvar` wait fails, it prints an error and returns early.
    fn wait_for_completion(&self) {
        let mut count = match self.count.lock() {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Failed to lock: {:?}", e);
                return;
            }
        };
        while *count > 0 {
            count = match self.condvar.wait(count) {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Failed to wait on condvar: {:?}", e);
                    return;
                }
            };
        }
    }
}

/// Represents a unique identifier for a task within a `TaskGraph`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TaskId(usize);

impl TaskId {
    /// Constructs a new `TaskId`.
    ///
    /// # Arguments
    ///
    /// * `id` - A usize value used as the unique identifier for the task.
    fn new(id: usize) -> Self {
        TaskId(id)
    }
}

/// Represents a task with a unique ID, a name, and an action to be executed.
struct Task {
    id: TaskId,
    name: String,                               // Name of the task
    action: Box<dyn FnOnce() + Send + 'static>, // The action associated with the task
}

/// A graph of tasks with dependencies.
///
/// Manages tasks, their dependencies, and execution in a thread pool.
pub struct TaskGraph {
    tasks: HashMap<TaskId, Task>,
    dependencies: HashMap<TaskId, Vec<TaskId>>,
    reverse_dependencies: HashMap<TaskId, Vec<TaskId>>,
    thread_pool: ThreadPool,
    next_id: usize, // Counter for generating unique task IDs
}

impl TaskGraph {
    /// Creates a new `TaskGraph` with a given thread pool.
    ///
    /// # Arguments
    ///
    /// * `thread_pool` - A ThreadPool for executing tasks.
    pub fn new(thread_pool: ThreadPool) -> Self {
        TaskGraph {
            tasks: HashMap::new(),
            dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            thread_pool,
            next_id: 0,
        }
    }

    /// Adds a task to the graph.
    ///
    /// # Arguments
    ///
    /// * `name` - A name for the task.
    /// * `action` - The action (closure) to be executed for the task.
    ///
    /// # Returns
    ///
    /// The unique `TaskId` of the added task.
    pub fn add_task<F>(&mut self, name: &str, action: F) -> TaskId
    where
        F: FnOnce() + Send + 'static,
    {
        let task_id = TaskId::new(self.next_id);
        self.next_id += 1;

        let task = Task {
            id: task_id,
            name: name.to_string(),
            action: Box::new(action),
        };

        self.tasks.insert(task_id, task);

        task_id
    }

    /// Adds a dependency between two tasks in the graph.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task that depends on `dependency_id`.
    /// * `dependency_id` - The ID of the task that `task_id` depends on.
    pub fn add_dependency(&mut self, task_id: TaskId, dependency_id: TaskId) {
        self.dependencies
            .entry(task_id)
            .or_insert_with(Vec::new)
            .push(dependency_id);
        self.reverse_dependencies
            .entry(dependency_id)
            .or_insert_with(Vec::new)
            .push(task_id);
        println!("{:?}", self.dependencies);
    }

    /// Performs a topological sort on the tasks based on their dependencies.
    ///
    /// # Returns
    ///
    /// A sorted vector of `TaskId`s representing the order in which tasks can be executed.
    fn topological_sort(&self) -> Vec<TaskId> {
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut sorted = Vec::new();

        for task in self.tasks.keys() {
            in_degree.insert(*task, 0);
        }

        for (&task, deps) in &self.dependencies {
            *in_degree.entry(task).or_insert(0) += deps.len();
        }

        for (&task, &count) in &in_degree {
            if count == 0 {
                queue.push_back(task);
            }
        }

        while let Some(task) = queue.pop_front() {
            sorted.push(task);

            if let Some(deps) = self.reverse_dependencies.get(&task) {
                for &dep in deps {
                    let count = in_degree.entry(dep).or_default();
                    *count -= 1;
                    if *count <= 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }

        sorted
    }

    /// Groups tasks into levels based on their dependencies.
    ///
    /// # Arguments
    ///
    /// * `sorted_tasks` - A vector of `TaskId`s sorted topologically.
    ///
    /// # Returns
    ///
    /// A vector of vectors, where each inner vector contains tasks at the same execution level.
    fn group_tasks(&self, sorted_tasks: Vec<TaskId>) -> Vec<Vec<TaskId>> {
        let mut levels: HashMap<TaskId, usize> = HashMap::new();
        let mut max_level = 0;

        for &task_id in &sorted_tasks {
            let level = self
                .dependencies
                .get(&task_id)
                .map(|deps| deps.iter().map(|dep| levels[dep] + 1).max().unwrap_or(0))
                .unwrap_or(0);
            levels.insert(task_id, level);
            max_level = max_level.max(level);
        }

        let mut grouped_tasks = vec![Vec::new(); max_level + 1];
        for (&task_id, &level) in &levels {
            grouped_tasks[level].push(task_id);
        }

        grouped_tasks
    }

    /// Executes all tasks in the graph according to their dependencies.
    ///
    /// Tasks are executed in parallel where possible, respecting their dependency constraints.
    pub fn execute(&mut self) {
        let sorted_tasks = self.topological_sort();
        let task_groups = self.group_tasks(sorted_tasks);

        for group in task_groups {
            let completion = Arc::new(TaskCompletion::new(group.len()));
            for task_id in group {
                if let Some(task) = self.tasks.remove(&task_id) {
                    let completion_clone = Arc::clone(&completion);
                    self.thread_pool.execute(move || {
                        (task.action)();
                        completion_clone.task_completed();
                    });
                }
            }
            completion.wait_for_completion();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::thread_pool::ThreadPool;

    use super::*;

    #[test]
    fn test_task_graph() {
        let pool = ThreadPool::new(4);
        let mut graph = TaskGraph::new(pool);

        let task_1 = graph.add_task("task1", Box::new(|| println!("execute task, 1")));
        let task_2 = graph.add_task("task2", Box::new(|| println!("execute task, 2")));
        let task_3 = graph.add_task("task3", Box::new(|| println!("execute task, 3")));
        let task_4 = graph.add_task("task4", Box::new(|| println!("execute task, 4")));
        let task_5 = graph.add_task("task5", Box::new(|| println!("execute task, 5")));

        graph.add_dependency(task_2, task_1);
        graph.add_dependency(task_3, task_2);
        graph.add_dependency(task_4, task_1);
        graph.add_dependency(task_5, task_3);
        graph.add_dependency(task_5, task_4);

        // タスクの実行
        graph.execute();
    }
}
