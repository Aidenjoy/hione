use hi_core::message::Message;
use hi_monitor::task_queue::TaskQueueMap;

#[test]
fn empty_queue_behavior() {
    let q = TaskQueueMap::new();
    assert_eq!(q.len("nobody"), 0);
    assert!(q.peek_next("nobody").is_none());
}

#[test]
fn enqueue_pop_fifo() {
    let mut q = TaskQueueMap::new();
    let a = Message::new_task("c", "opencode", "A");
    let b = Message::new_task("c", "opencode", "B");
    q.enqueue("opencode", a.clone());
    q.enqueue("opencode", b.clone());
    assert_eq!(q.len("opencode"), 2);

    let peeked = q.peek_next("opencode").unwrap();
    assert_eq!(peeked.id, a.id);

    let popped = q.pop_next("opencode").unwrap();
    assert_eq!(popped.id, a.id);
    assert_eq!(q.len("opencode"), 1);

    let popped = q.pop_next("opencode").unwrap();
    assert_eq!(popped.id, b.id);
    assert_eq!(q.len("opencode"), 0);
    assert!(q.pop_next("opencode").is_none());
}

#[test]
fn cancel_removes_matching_task() {
    let mut q = TaskQueueMap::new();
    let a = Message::new_task("c", "opencode", "A");
    let b = Message::new_task("c", "opencode", "B");
    let c = Message::new_task("c", "opencode", "C");
    q.enqueue("opencode", a.clone());
    q.enqueue("opencode", b.clone());
    q.enqueue("opencode", c.clone());

    q.cancel("opencode", b.id);
    assert_eq!(q.len("opencode"), 2);

    let first = q.pop_next("opencode").unwrap();
    assert_eq!(first.id, a.id);
    let second = q.pop_next("opencode").unwrap();
    assert_eq!(second.id, c.id);
}

#[test]
fn cancel_unknown_target_is_noop() {
    let mut q = TaskQueueMap::new();
    q.cancel("ghost", uuid::Uuid::new_v4());
    assert_eq!(q.len("ghost"), 0);
}

#[test]
fn queues_are_per_target() {
    let mut q = TaskQueueMap::new();
    let a = Message::new_task("c", "opencode", "A");
    let b = Message::new_task("c", "gemini", "B");
    q.enqueue("opencode", a);
    q.enqueue("gemini", b);
    assert_eq!(q.len("opencode"), 1);
    assert_eq!(q.len("gemini"), 1);
}

#[test]
fn default_equivalent_to_new() {
    let q: TaskQueueMap = Default::default();
    assert_eq!(q.len("anything"), 0);
}
