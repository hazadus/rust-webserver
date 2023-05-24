use std::{
    sync::{ mpsc, Arc, Mutex },
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

// Type aliases allow us to make long types shorter for ease of use:
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new `ThreadPool`.
    ///
    /// The `size` is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // We put the receiver in an `Arc` and a `Mutex`. For each new worker, we clone
            // the `Arc` to bump the reference count so the workers can share ownership of the
            // `receiver`.
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// We can be confident that `FnOnce` is the trait we want to use because the thread for
    /// running a request will only execute that request’s closure one time, which matches the Once
    /// in `FnOnce`.
    ///
    /// The `F` type parameter also has the trait bound Send and the lifetime bound `'static`, which
    /// are useful in our situation: we need `Send` to transfer the closure from one thread to
    /// another and `'static` because we don’t know how long the thread will take to execute.
    ///
    /// We still use the `()` after `FnOnce` because this FnOnce represents a closure that takes no
    /// parameters and returns the unit type `()`. Just like function definitions, the return type
    /// can be omitted from the signature, but even if we have no parameters, we still need the
    /// parentheses.
    pub fn execute<F>(&self, f: F)
    where F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

/// The Worker picks up code that needs to be run and runs the code in the Worker’s thread.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Here, we first call `lock` on the receiver to acquire the mutex, and then we call
            // `unwrap` to panic on any errors. Acquiring a lock might fail if the mutex is in a
            // poisoned state, which can happen if some other thread panicked while holding the
            // lock rather than releasing the lock. In this situation, calling `unwrap` to have this
            // thread panic is the correct action to take.
            //
            // If we get the lock on the mutex, we call `recv` to receive a `Job` from the channel.
            // A final `unwrap` moves past any errors here as well, which might occur if the thread
            // holding the sender has shut down, similar to how the `send` method returns `Err`
            // if the `receiver` shuts down.
            //
            // The call to `recv` blocks, so if there is no job yet, the current thread will wait
            // until a job becomes available. The `Mutex<T>` ensures that only one `Worker` thread
            // at a time is trying to request a job.
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker { id, thread }
    }
}