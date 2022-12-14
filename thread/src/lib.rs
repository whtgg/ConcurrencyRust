#![feature(local_key_cell_methods)]
use std::{thread, time, cell:: RefCell, sync::{atomic::{AtomicBool, AtomicI64}, Arc, mpsc::channel}, vec, rc::Rc, ops::Deref};

use crossbeam::sync::Parker;
use send_wrapper::SendWrapper;
use thread_control::make_pair;
use thread_priority::{ ThreadPriority, ThreadBuilder};

/// 分配一个线程
pub fn start_one_thread() {
    // 获得当前运行程序并行度
    let count =  thread::available_parallelism().unwrap().get();
    println!("available_parallelism {}",count);
    // 分配线程
    let join_handle = thread::spawn(|| {
        println!("spawn a thread");
    }); 
    // 执行线程(如果不执行这个方法,上面分配的线程不执行)
    // join_handle.join().unwrap();
    match join_handle.join() {
        Ok(data) => {
            println!("thread return data:{:?}",data);
        },
        Err(e) => {
            println!("thread return error:{:?}",e);
        },
    }
}

/// 分配多个thread
pub fn start_many_thread() {
    let n = 10;
    
    // 分配多个线程 
    let join_handles = (0..n).map(|i| {
       thread::spawn(move || {
            println!("spawn {} thread",i);
       })
    });

    // 获取执行线程返回结果
    for join_handle in join_handles {
        match join_handle.join() {
            Ok(data) => {
                println!("thread return data:{:?}",data);
            },
            Err(e) => {
                println!("thread return error:{:?}",e);
            },
        }
    }
}


// 获取当前线程信息
pub fn current_thread() {
    let current_thread = thread::current();
    println!("current thread ID {:?}",current_thread.id());
    // 自定义线程信息
    let build_thread = thread::Builder::new()
        .name(String::from("A"))
        .stack_size(1024)
        .spawn(|| {
            let current_thread = thread::current();
            println!("current thread ID:{:?}-Name:{:?}",current_thread.id(),current_thread.name());
        })
        .unwrap();
    build_thread.join().unwrap();
}

/// 线程让出时间片
pub fn thread_yield() {
    let join_handle1 = thread::spawn(|| {
       println!("thread1");
       thread::sleep(time::Duration::from_secs(2));
       thread::yield_now();
       println!("yield_now_thread1"); 
    });
    
    let join_handle2 = thread::spawn(|| {
        println!("thread2");
        thread::sleep(time::Duration::from_secs(2));
        thread::yield_now();
        println!("yield_now_thread2"); 
    });

    join_handle1.join().unwrap();
    join_handle2.join().unwrap();
}

/// 线程执行优先级
/// ThreadPriority::Max在mac下报错
pub fn thread_priority() {
    let join_handle1 = ThreadBuilder::default()
        .name("thread1")
        .priority(ThreadPriority::Min)
        .spawn(|result| {
            assert!(result.is_ok());
        })
        .unwrap();

    let join_handle2 = ThreadBuilder::default()
        .name("thread2")
        .priority(ThreadPriority::Min)
        .spawn(|result| {
            assert!(result.is_ok());
        })
        .unwrap();
    
    join_handle1.join().unwrap();
    join_handle2.join().unwrap();
}

/// 线程传值
/// 已经移动所有权其他线程不可用
pub fn thread_move() {
    let data = vec![1,2,3];

    let join_handle1 = thread::spawn( move || {
        println!("move data: {:?}",data);
    });

    join_handle1.join().unwrap();

    // let join_handle2 = thread::spawn( move || {
    //     println!("move data: {:?}",data);
    // });

    // join_handle2.join().unwrap();
}

/// thread local
/// 但是thread local包裹变量有如下好处
/// 1. 可以重复移动到多个线程
/// 2. 线程内修改变量值只在当前线程有效
/// 需要注意 不能给thread_local变量实现Drop
pub fn thread_with_thread_local() {
    thread_local!(static FOO: RefCell<u32> = RefCell::new(1));

    FOO.with(|f| {
        assert_eq!(*f.borrow(), 1);
        *f.borrow_mut() = 2;
    });

   
    let t = thread::spawn(move|| {
        FOO.with(|f| {
            assert_eq!(*f.borrow(), 1);
            *f.borrow_mut() = 3;
        });
    });

    let t1 = thread::spawn(move|| {
        FOO.with(|f| {
            assert_eq!(*f.borrow(), 1);
            *f.borrow_mut() = 4;
        });
    });

    t.join().unwrap();
    t1.join().unwrap();

    FOO.with(|f| {
        assert_eq!(*f.borrow(), 2);
    });
    println!("{}",FOO.take());

}


pub fn thread_without_thread_local() {
    // a未实现Copy 所以两次move会失败
    // let mut a = vec![1,3,5];
    // let mut b = 0;
    
    // thread::spawn(move || {
    //     println!("hello from the  local");
    //     dbg!(&a);
    // });

    // thread::spawn(move || {
    //     println!("hello from the  local");
    //     b += a[0] + a[1];
    // });

    // a.push(4);
    // println!("{:?}",a);
}

//  thread::scope可以使用上下文变量 不用获取所有权
//  子线程不能实现孙线程
pub fn thread_without_thread_scoped() {
    let mut a = vec![1,3,5];
    let mut b = 0;
    
    thread::scope(|s| {
        s.spawn(|| {
            println!("hello from the  local");
            dbg!(&a);
        });

        s.spawn(|| {
            println!("hello from the  local");
            b += a[0] + a[1];
        });
    });
    a.push(4);
    println!("{:?}",a);
}


pub fn thread_park() {
    // Basic
    // let handle = thread::spawn(|| {
    //     thread::sleep(time::Duration::from_secs(1));
    //     thread::park();
    //     println!("Hello from a park thread")
    // });

    // handle.thread().unpark();
    // handle.join().unwrap();
    
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);
    
    let thread_handle = thread::spawn(move || {
        while !flag_clone.load(std::sync::atomic::Ordering::Acquire) {
            println!("Parking thread");
            thread::park_timeout(time::Duration::from_secs(1));
            println!("thread unpark");
        }
        println!("thread Received");
    });

    thread::sleep(time::Duration::from_secs(10));

    flag.store(true, std::sync::atomic::Ordering::Release);
    thread_handle.thread().unpark();
    thread_handle.join().unwrap();
    
}


/// 子线程可以创建孙线程
pub fn crossbeam_scope() {
    let mut a = vec![1,3,5];
    let mut b = 0;
    
    crossbeam::scope(|s| {
        s.spawn(|_| {
            println!("hello from the  local");
            dbg!(&a);
            // 还可以创建子线程
           //  ss.spawn(f)
        });

        s.spawn(|_| {
            println!("hello from the  local");
            b += a[0] + a[1];
        });
    }).unwrap();
    a.push(4);
    println!("{:?}",a);
}

// 子线程也可以创建孙线程
pub fn rayon_scope() {
    let mut a = vec![1,3,5];
    let mut b = 0;
    
    rayon::scope(|s| {
        s.spawn(|_| {
            println!("hello from the  local");
            dbg!(&a);
        });

        s.spawn(|_| {
            println!("hello from the  local");
            b += a[0] + a[1];
        });
    });
    a.push(4);
    println!("{:?}",a);
}

pub fn arc_send() {
    let counter = Arc::new(42);

    let (sender, receiver) = channel();

    let _t = thread::spawn(move || {
        sender.send(counter).unwrap();
    });

    let value = receiver.recv().unwrap();

    println!("received from the main thread: {}", value);
}

/// send_wrapper 可以让任何没有实现Send+Sync的值可以在多线程中移动
/// 必须保证在声明值的线程访问值
pub fn wrapper_send() {
    let a  = SendWrapper::new(Rc::new(false));
    let (st,rt) = channel();
    thread::spawn(move|| {
       //  println!("{}",a.deref()); // panic 必须在声明线程访问值
        st.send(a).unwrap();
    });

    let value = rt.recv().unwrap();
    println!("{}",value.deref());
}


/// 根据flag查看状态
/// 根据control控制状态
pub fn thread_control_exec() {
    let (flag,control) = make_pair();
    let handle =  thread::spawn(move || {
        if flag.alive() {
            println!("{}",flag.is_alive());
        }
    });
    control.interrupt();
    handle.join().unwrap();
    println!("{}",control.is_interrupted());
}

// #[cfg(not(target_os = "macos"))]
// pub fn use_affinity() {
//     // Select every second core
//     let cores: Vec<usize> = (0..get_core_num()).step_by(2).collect();
//     println!("Binding thread to cores : {:?}", &cores);

//     affinity::set_thread_affinity(&cores).unwrap();
//     println!(
//         "Current thread affinity : {:?}",
//         affinity::get_thread_affinity().unwrap()
//     );
// }

pub fn go_style_spawn() {
    use go_spawn::go;

    let counter = Arc::new(AtomicI64::new(0));
    let copy_of_counter = counter.clone();

    go! {
        for _ in 0..1_000_000 {
            copy_of_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

//  p.unparker如果不克隆总是报错 目前不知道怎么解决
pub fn thread_parking() {
    let p = Parker::new();
    let u = p.unparker().clone();

    // Notify the parker.
    u.unpark();
    
    // Wakes up immediately because the parker is notified.
    p.park();
    thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(500));
        u.unpark();
    });
    // Wakes up when `u.unpark()` notifies and then goes back into unnotified state.
    p.park();
}