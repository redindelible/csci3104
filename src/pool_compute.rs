use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;


pub fn pool_compute<I, F, O>(inputs: I, num_workers: usize, compute: F) -> Vec<O>
    where I: Iterator + Send, <I as Iterator>::Item: Send, O: Send, F: (Fn(I::Item) -> O) + Send + Copy {
    let (input_sender, input_recv) = channel();
    let (result_sender, results_recv) = channel();
    let input_recv = Arc::new(Mutex::new(input_recv));

    thread::scope(|s| {
        for _ in 0..num_workers {
            let input_recv = Arc::clone(&input_recv);
            let result_sender = result_sender.clone();

            s.spawn(move || {
                loop {
                    let next = input_recv.lock().unwrap().recv();
                    match next {
                        Ok(val) => {
                            let res = compute(val);
                            result_sender.send(res).unwrap();
                        },
                        Err(_) => break
                    }
                }
            });
        }

        inputs.for_each(|item| input_sender.send(item).unwrap());
        drop(input_sender);
    });
    drop(result_sender);

    results_recv.iter().collect()
}