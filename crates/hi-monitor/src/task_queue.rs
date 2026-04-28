use hi_core::message::Message;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

pub struct TaskQueueMap {
    queues: HashMap<String, VecDeque<Message>>,
}

impl TaskQueueMap {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    pub fn enqueue(&mut self, target: &str, msg: Message) {
        self.queues
            .entry(target.to_string())
            .or_default()
            .push_back(msg);
    }

    pub fn peek_next(&self, target: &str) -> Option<&Message> {
        self.queues.get(target)?.front()
    }

    pub fn pop_next(&mut self, target: &str) -> Option<Message> {
        self.queues.get_mut(target)?.pop_front()
    }

    pub fn cancel(&mut self, target: &str, task_id: Uuid) {
        if let Some(queue) = self.queues.get_mut(target) {
            queue.retain(|m| m.id != task_id);
        }
    }

    pub fn len(&self, target: &str) -> usize {
        self.queues.get(target).map(|q| q.len()).unwrap_or(0)
    }
}

impl Default for TaskQueueMap {
    fn default() -> Self {
        Self::new()
    }
}
