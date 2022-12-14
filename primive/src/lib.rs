#![feature(sync_unsafe_cell)]

use std::{sync::{Arc, Barrier, atomic::{ Ordering, AtomicUsize}, Condvar, Mutex, Once, RwLock, mpsc::{channel, sync_channel}}, thread, time, hint, cell::SyncUnsafeCell};

/// barrier 会阻塞为序列执行
pub fn p_barrier() {
    let barrier = Arc::new(Barrier::new(1));
    
    let mut handles = vec![];
    
    for _ in 0..10  {
        let barrier = barrier.clone();
        handles.push(thread::spawn( move || {
            println!("barrier before");
            thread::sleep(time::Duration::from_secs(1));
            barrier.wait();
            println!("barrier after");
        }));
    }
    barrier.wait();
    // println!("{}",handles.len()); // 如果不join输出会乱序
    handles.into_iter().for_each(|f| {
        f.join().unwrap();
    });
}

/// Rust原子操作
/// 原子内存顺序(Ordering)
/// 参考:https://zhuanlan.zhihu.com/p/365905573
/// Relaxed 这是最宽松的规则，它对编译器和 CPU 不做任何限制，可以乱序
/// Release 当我们写入数据（上面的 store）的时候，如果用了 Release order，那么
/// 对于当前线程，任何读取或写入操作都不能被被乱序排在这个 store 之后
/// 对于其它线程，如果它们使用了 Acquire 来读取这个 atomic 的数据， 那么它们看到的是修改后的结果
/// Acquire 当我们读取数据的时候，如果用了 Acquire order，那么
/// 对于当前线程，任何读取或者写入操作都不能被乱序排在这个读取之前
/// 对于其它线程，如果使用了 Release 来修改数据，那么，修改的值对当前线程可见
/// AcqRel Acquire 和 Release 的结合，同时拥有 Acquire 和 Release 的保证。这个一般用在 fetch_xxx 上，比如你要对一个 atomic 自增 1，你希望这个操作之前和之后的读取或写入操作不会被乱序，并且操作的结果对其它线程可见;
/// SeqCst 最严格的 ordering，除了 AcqRel 的保证外，它还保证所有线程看到的所有的 SeqCst 操作的顺序是一致的
pub fn p_atomic() {
    
    let spin_lock = Arc::new(AtomicUsize::new(0));
    let spin_lock_clone = Arc::clone(&spin_lock);
    
    let handle = thread::spawn(move|| {
        spin_lock_clone.store(1, Ordering::SeqCst);
    });

    // 如果没有设置值 则Cpu空旋
    while spin_lock.load(Ordering::SeqCst) != 1 {
        hint::spin_loop();
    }
    match handle.join() {
        Ok(_) => {
            println!("{}",spin_lock.load(Ordering::SeqCst));
        },
        Err(e) => {
            println!("{:?}",e);
        }
    }
}

/// condvar + mutex 可以实现线程间通信
pub fn p_condvar() {
    let cond_var = Arc::new(Condvar::new());
    let mux =Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let p_cond_var_clone = cond_var.clone();
        let p_mux_clone = mux.clone();
        let handle = thread::spawn(move || {
           let mut num =  p_mux_clone.lock().unwrap();
           *num += 1;
        //    println!("num1 {}",num);
           p_cond_var_clone.notify_one();
        });
        handles.push(handle);
    }


    for _ in 0..10 {
        let c_cond_var_clone = cond_var.clone();
        let c_mux_clone = mux.clone();
        handles.push(thread::spawn(move || {
            let mut num =  c_mux_clone.lock().unwrap();
            while *num == 0 {
                num = c_cond_var_clone.wait(num).unwrap();
            }
            *num += 1;
            println!("num2 {}",*num-1);
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("{}",*mux.lock().unwrap());
}

/// 线程间锁
pub fn p_mutex() {
    // let lock = Arc::new(Mutex::new(0_u32));
    // let lock2 = Arc::clone(&lock);

    // let _ = thread::spawn(move || -> () {
    //     let _guard = lock2.lock().unwrap();

    //     // 在拿锁过程中发生panic
    //     panic!();
    // })
    // .join();

    // // 异常传递重新获得锁
    // let mut guard = match lock.lock() {
    //     Ok(guard) => guard,
    //     Err(poisoned) => {
    //         println!("error");
    //         poisoned.into_inner()
    //     },
    // };


    let mux_data = Arc::new(Mutex::new(vec![1,2,3,4]));
    let mux_res = Arc::new(Mutex::new(0));
    let barrier = Arc::new(Barrier::new(0));

    let mut threads = Vec::with_capacity(3);

    (0..3).for_each(|_| {
        let mux_data_clone = Arc::clone(&mux_data);
        let mux_res_clone = Arc::clone(&mux_res);
        let barrier_clone = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            let mut data = mux_data_clone.lock().unwrap();
            let result = data.iter().fold(0,|acc,x|{acc + 2*x});
            data.push(result);
            *mux_res_clone.lock().unwrap() = result;
            drop(data);
            drop(result);
            barrier_clone.wait();
        }));
    });

    threads.into_iter().for_each(|t| {
        t.join().unwrap();
    });
    println!("{}",mux_res.lock().unwrap());
}

/// 只执行一次 可以用于初始化
pub fn p_once() {
    static mut VAL:usize = 10;
    let onece = Once::new();
    onece.call_once(|| {
        // VAL = get_once_val();
        unsafe {
             VAL = get_once_val();
        }
     });

    if onece.is_completed() {
        println!("completed");
    }
}

/// once值
fn get_once_val() -> usize {
    12
}

/// 读写锁
pub fn p_rwlock() {
    let rwlock = Arc::new(RwLock::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let rwlock_clone = Arc::clone(&rwlock);
        handles.push(thread::spawn(move || {
            let num = rwlock_clone.read().unwrap();
            println!("num -- {}",*num);
        }));
    }

    for i in 0..10 {
        let rwlock_clone = Arc::clone(&rwlock);
        handles.push(thread::spawn(move || {
            let mut num = rwlock_clone.write().unwrap();
            *num = i;
        }));
    }

    for _ in 0..10 {
        let rwlock_clone = Arc::clone(&rwlock);
        handles.push(thread::spawn(move || {
            let num = rwlock_clone.read().unwrap();
            println!("num -- --  {}",*num);
        }));
    }
    handles.into_iter().for_each(|h|{
        h.join().unwrap();
    });
    println!("final {}",*rwlock.read().unwrap());
}

/// channel
pub fn p_mpsc() {
   let (tx,rx) = channel();
   let mut  handles = vec![];
   
   for i in 0..10 {
        let tx_clone  = tx.clone();
        handles.push(thread::spawn(move || {
            tx_clone.send(i).unwrap();
        }));
   }
   for handle in handles {
        handle.join().unwrap();
   }
   // 需要手动drop 不然会一直后台运行
   drop(tx);
   for i in rx {
        println!("channel i {}",i);
   }
}

/// sync 通道存入的值必须取出来才可以放进下一个值
/// 不然会一直堵塞 类似go单值channel
/// rust默认channel是多生产者
pub fn p_mpsc_async() {
    let (tx,rx)  = sync_channel::<usize>(0);
   //let (tx,rx)  = channel::<usize>();
    tx.send(1).unwrap();
    thread::spawn(move || {
        tx.send(9).unwrap();
    });
    println!("{}",rx.recv().unwrap());
    println!("{}",rx.recv().unwrap());
}

/// 线程间安全传值
pub fn p_arc() {
    let val = Arc::new(SyncUnsafeCell::new(3));
    let val2 = &val.clone();

    let mut handles = vec![];

    for _ in 0..10 {
        let val = Arc::clone(&val);

        let handle = thread::spawn(move || {
            let v = val.get();
            unsafe {*v += *v};
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    unsafe {
        let v = val2.get().read();
        println!("SyncUnsafeCell: {:?}",  v);
    }
}