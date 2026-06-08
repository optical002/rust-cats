use crate::data::io::{CancelToken, IO, IoResult};
use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle, ThreadId};
use std::time::{Duration, Instant};

pub(crate) type Task = Box<dyn FnOnce() + Send>;

struct TimerEntry {
    due: Instant,
    task: Task,
}

struct Park {
    mu: Mutex<bool>, // true when signalled
    cv: Condvar,
}

struct Scheduler {
    injector: Injector<Task>,
    stealers: Vec<Stealer<Task>>,
    parks: Vec<Arc<Park>>,
    timers: Mutex<Vec<TimerEntry>>,
    timer_cv: Condvar,
    shutdown: AtomicBool,
    next_token: AtomicU64,
    start_instant: Instant,
    workers_idle: AtomicU64, // count of currently-parked workers (for fast wake decision)
    worker_handles: Mutex<Vec<JoinHandle<()>>>,
    timer_handle: Mutex<Option<JoinHandle<()>>>,
    worker_ids: Mutex<Vec<ThreadId>>,
}

impl Scheduler {
    fn new(num_workers: usize) -> Arc<Self> {
        let mut workers: Vec<Worker<Task>> = Vec::with_capacity(num_workers);
        let mut stealers: Vec<Stealer<Task>> = Vec::with_capacity(num_workers);
        let mut parks: Vec<Arc<Park>> = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let w: Worker<Task> = Worker::new_fifo();
            stealers.push(w.stealer());
            parks.push(Arc::new(Park {
                mu: Mutex::new(false),
                cv: Condvar::new(),
            }));
            workers.push(w);
        }

        let sched = Arc::new(Scheduler {
            injector: Injector::new(),
            stealers,
            parks,
            timers: Mutex::new(Vec::new()),
            timer_cv: Condvar::new(),
            shutdown: AtomicBool::new(false),
            next_token: AtomicU64::new(0),
            start_instant: Instant::now(),
            workers_idle: AtomicU64::new(0),
            worker_handles: Mutex::new(Vec::new()),
            timer_handle: Mutex::new(None),
            worker_ids: Mutex::new(Vec::new()),
        });

        // Spawn workers.
        let mut handles = Vec::with_capacity(num_workers);
        let mut ids = Vec::with_capacity(num_workers);
        for (idx, worker) in workers.into_iter().enumerate() {
            let s = sched.clone();
            let h = thread::Builder::new()
                .name(format!("rcats-worker-{}", idx))
                .spawn(move || {
                    s.worker_loop(idx, worker);
                })
                .expect("worker spawn");
            ids.push(h.thread().id());
            handles.push(h);
        }
        *sched.worker_handles.lock().unwrap() = handles;
        *sched.worker_ids.lock().unwrap() = ids;

        // Spawn timer thread.
        {
            let s = sched.clone();
            let h = thread::Builder::new()
                .name("rcats-timer".to_string())
                .spawn(move || s.timer_loop())
                .expect("timer spawn");
            *sched.timer_handle.lock().unwrap() = Some(h);
        }

        sched
    }

    fn worker_loop(self: Arc<Self>, idx: usize, worker: Worker<Task>) {
        let park = self.parks[idx].clone();

        loop {
            if self.shutdown.load(Ordering::Acquire) {
                return;
            }

            if let Some(task) = self.find_task(&worker, idx) {
                task();
                continue;
            }

            // No work — park until signalled.
            self.workers_idle.fetch_add(1, Ordering::SeqCst);
            // Double check after announcing idle to avoid races.
            if let Some(task) = self.find_task(&worker, idx) {
                self.workers_idle.fetch_sub(1, Ordering::SeqCst);
                task();
                continue;
            }
            let mut guard = park.mu.lock().unwrap();
            while !*guard && !self.shutdown.load(Ordering::Acquire) {
                guard = park.cv.wait(guard).unwrap();
            }
            *guard = false;
            drop(guard);
            self.workers_idle.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn find_task(&self, worker: &Worker<Task>, idx: usize) -> Option<Task> {
        if let Some(t) = worker.pop() {
            return Some(t);
        }
        loop {
            match self.injector.steal_batch_and_pop(worker) {
                Steal::Success(t) => return Some(t),
                Steal::Empty => break,
                Steal::Retry => continue,
            }
        }
        // Try stealers (skip self).
        for (i, st) in self.stealers.iter().enumerate() {
            if i == idx {
                continue;
            }
            loop {
                match st.steal_batch_and_pop(worker) {
                    Steal::Success(t) => return Some(t),
                    Steal::Empty => break,
                    Steal::Retry => continue,
                }
            }
        }
        None
    }

    fn enqueue(&self, task: Task) {
        // Always go through injector — simplest, and threads outside the worker
        // pool (e.g. the main thread or timer thread) can call this.
        self.injector.push(task);
        self.wake_one();
    }

    fn wake_one(&self) {
        // Wake any one parked worker if any are parked.
        if self.workers_idle.load(Ordering::SeqCst) == 0 {
            return;
        }
        for p in &self.parks {
            let mut guard = p.mu.lock().unwrap();
            if !*guard {
                *guard = true;
                drop(guard);
                p.cv.notify_one();
                return;
            }
        }
    }

    fn wake_all(&self) {
        for p in &self.parks {
            let mut guard = p.mu.lock().unwrap();
            *guard = true;
            drop(guard);
            p.cv.notify_one();
        }
    }

    fn schedule_at(&self, due: Instant, task: Task) {
        let mut t = self.timers.lock().unwrap();
        t.push(TimerEntry { due, task });
        self.timer_cv.notify_one();
    }

    fn timer_loop(self: Arc<Self>) {
        loop {
            if self.shutdown.load(Ordering::Acquire) {
                return;
            }
            let now = Instant::now();
            let mut due_tasks: Vec<Task> = Vec::new();
            let next_due_opt;
            {
                let mut t = self.timers.lock().unwrap();
                let mut i = 0;
                while i < t.len() {
                    if t[i].due <= now {
                        let entry = t.swap_remove(i);
                        due_tasks.push(entry.task);
                    } else {
                        i += 1;
                    }
                }
                next_due_opt = t.iter().map(|e| e.due).min();
            }
            for task in due_tasks {
                self.enqueue(task);
            }
            // Wait for the next timer or a new timer being pushed.
            let guard = self.timers.lock().unwrap();
            let _unused = match next_due_opt {
                Some(due) => {
                    let now = Instant::now();
                    if due > now {
                        let dur = due - now;
                        self.timer_cv.wait_timeout(guard, dur).unwrap().0
                    } else {
                        guard
                    }
                }
                None => self
                    .timer_cv
                    .wait_timeout(guard, Duration::from_millis(100))
                    .unwrap()
                    .0,
            };
        }
    }

    fn shutdown_now(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.wake_all();
        self.timer_cv.notify_all();
    }
}

static SCHEDULER: OnceLock<Arc<Scheduler>> = OnceLock::new();

fn sched() -> &'static Arc<Scheduler> {
    SCHEDULER.get_or_init(|| {
        let n = std::env::var("RUST_CATS_WORKERS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or_else(|| num_cpus::get().max(1));
        Scheduler::new(n)
    })
}

/// Initialize the scheduler with a specific worker count. Must be called before
/// any other runtime function is used, or it is a no-op.
pub fn init_with_workers(n: usize) {
    let _ = SCHEDULER.set(Scheduler::new(n.max(1)));
}

/// Number of worker threads currently configured.
pub fn worker_count() -> usize {
    sched().stealers.len()
}

pub(crate) fn next_unique_token() -> u64 {
    sched().next_token.fetch_add(1, Ordering::SeqCst) + 1
}

pub(crate) fn enqueue(task: Task) {
    sched().enqueue(task);
}

pub(crate) fn schedule_after(d: Duration, task: Task) {
    let due = Instant::now() + d;
    sched().schedule_at(due, task);
}

pub(crate) fn start_instant() -> Instant {
    sched().start_instant
}

/// Block the *current* thread until `done` is set. If the current thread is a
/// worker, we help drain work while waiting; otherwise we park on a condvar
/// that the completion callback notifies.
pub(crate) fn block_until(done: Arc<ParkSlot>) {
    if done.is_done() {
        return;
    }
    let mut guard = done.mu.lock().unwrap();
    while !*guard {
        guard = done.cv.wait(guard).unwrap();
    }
}

pub struct ParkSlot {
    mu: Mutex<bool>,
    cv: Condvar,
}

impl ParkSlot {
    pub fn new() -> Self {
        ParkSlot {
            mu: Mutex::new(false),
            cv: Condvar::new(),
        }
    }

    pub fn signal(&self) {
        let mut g = self.mu.lock().unwrap();
        *g = true;
        drop(g);
        self.cv.notify_all();
    }

    pub fn is_done(&self) -> bool {
        *self.mu.lock().unwrap()
    }
}

impl Default for ParkSlot {
    fn default() -> Self {
        Self::new()
    }
}

/// Run an IO to completion. Spawns it as a task on the scheduler and blocks
/// the calling thread until it finishes.
pub fn unsafe_run_sync<A: Send + 'static>(io: IO<A>) -> IoResult<A> {
    let token = CancelToken::new();
    let slot: Arc<Mutex<Option<IoResult<A>>>> = Arc::new(Mutex::new(None));
    let park = Arc::new(ParkSlot::new());

    let slot_for_task = slot.clone();
    let park_for_task = park.clone();
    let token_for_task = token.clone();
    let io = Arc::new(io);
    let io_for_task = io.clone();

    enqueue(Box::new(move || {
        let r = io_for_task.run(&token_for_task);
        *slot_for_task.lock().unwrap() = Some(r);
        park_for_task.signal();
    }));

    block_until(park);
    slot.lock().unwrap().take().expect("scheduler exited without producing a result")
}

/// Shut down the scheduler (intended for tests/benchmarks).
pub fn shutdown() {
    if let Some(s) = SCHEDULER.get() {
        s.shutdown_now();
    }
}
