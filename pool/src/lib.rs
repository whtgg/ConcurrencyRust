use std::{usize, sync::{mpsc::channel, atomic::{AtomicUsize, Ordering}, Arc, Barrier}, time};

use fast_threadpool::{ThreadPool, ThreadPoolConfig};
use threadpool::ThreadPool as Pool;

fn fib(num:usize) -> usize{
    if num == 0 || num == 1 {
        return num;
    }
    let (a,b) = rayon::join(|| fib(num-1), || fib(num-2));
    return a + b;
}


pub fn rayon_pool() {
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(5)
        .build()
        .unwrap();
    let n = thread_pool.install(||fib(30));
    println!("{}",n);
}

/// threadpool 并行库
pub fn threadpool_pool() {
    let jobs = 4;
    let thread_pool = Pool::new(5);
    let (tx,rx) = channel();
    for i in 0..jobs {
        let txx  = tx.clone();
        thread_pool.execute(move|| {
            txx.send(i).unwrap();
        });
    }

    // for r in rx.iter() {
    //     println!("{}",r);
    // }
    println!("{}",rx.iter().take(jobs).fold(0, |a,b| a +b));
}


/// workers必须大于等于jobs 否则会发生死锁
pub fn threadpool_basic() {
    // let pool = ThreadPool::new(5);
    
    // for _ in 0..10 {
    //     pool.execute(|| {
    //         println!("execute from pool");
    //     });
    // }
    // pool.join();

    let workers = 23;
    let jobs = 23;
    let pool = Pool::new(workers);
    let atomics = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(jobs + 1));
    
    for _ in 0..jobs {
        let atomics = atomics.clone();
        let barrier  = barrier.clone();
        pool.execute( move || {
            atomics.fetch_add(1, Ordering::Relaxed);
            barrier.wait();
        });
    }
    barrier.wait();
    println!("{}",atomics.load(Ordering::SeqCst));

}

/// into_sync_handler 同步
/// into_async_handler 异步
pub fn fast_thread_pool() {
    let pool  = ThreadPool::start(ThreadPoolConfig::default(), ())
        .into_sync_handler();
    let result = pool.execute(|_| { 4 + 4}).unwrap();
    println!("{}",result);
}

pub fn scoped_pool() {
    let workers = 4;
    let mut pool = scoped_threadpool::Pool::new(workers);
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6, 7];
    pool.scoped(|scope| {
        for e in &mut vec {
            // ... and add 1 to it in a seperate thread
            scope.execute(move || {
                *e += 1;
            });
        }
    });
    println!("{:?}",vec);
}

pub fn scheduled_pool() {
   let (tx,rx) =  channel();
   let pool = scheduled_thread_pool::ScheduledThreadPool::new(4);
   
   let _ = pool.execute_after(time::Duration::from_secs(2), move || {
        tx.send(1).unwrap();
   });
   
   rx.iter().for_each(|r| {
        println!("{}",r);
   });
}