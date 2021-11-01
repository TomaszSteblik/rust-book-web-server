use std::{sync::{Arc, Mutex, mpsc}, thread};

struct Worker{
    id : i64,
    thread : Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id_parameter: i64, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker{

        

        Worker{
            id: id_parameter,
            thread : Some(thread::spawn(move || -> () {
                loop {
                    let msg = receiver.lock().unwrap().recv().unwrap();

                    match msg {
                        Message::NewJob(job) => {
                            println!("Worker {} got a job; executing.", id_parameter);
                            job();
                        }
                        Message::Terminate => {
                            println!("Worker {} was told to terminate.", id_parameter);

                            break;
                        }
                    }
                }
            }))
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate
}

pub struct ThreadPool{
    workers : Vec<Worker>,
    sender : mpsc::Sender<Message>
}

impl ThreadPool {
    pub fn new(threads_amount: usize) -> ThreadPool{

        assert!(threads_amount > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(threads_amount);
        for i in  0..threads_amount{
            workers.push(Worker::new(i as i64, Arc::clone(&receiver)))
        }

    

        ThreadPool{
            workers,
            sender
        }
    }

    pub fn execute<F>(&self, closure: F)
    where F: FnOnce() + 'static + Send{

        let job = Box::new(closure);

        self.sender.send(Message::NewJob(job)).unwrap();

    }
}

impl Drop for ThreadPool{
    fn drop(&mut self) {

        println!("Sending terminate message to all workers");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        
        println!("Shutting down all workers.");

        for worker in &mut self.workers {

            println!("Shutting down worker {}", worker.id);

            if let Some(thr) = worker.thread.take(){
                thr.join().unwrap();
            }

        }
    }
}