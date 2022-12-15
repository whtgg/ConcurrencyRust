use std::{sync::{Mutex, Condvar, Arc}, thread};


#[derive(Debug,Default)]
struct Buffer {
    // data:Mutex<Option<i32>>,
    // is_empty: Condvar,
    // is_full: Condvar,
    data: Mutex<Option<i32>>,
    data_cv: Condvar,
}

impl Buffer {
    // pub fn insert(&self,val:i32){
    //     let mut lock = self.data.lock().unwrap();
    //     println!("insert {:?}",lock);
    //     while lock.is_some() {
    //         lock = self.is_full.wait(lock).unwrap();
    //         self.is_full.notify_one();
    //     }
    //     *lock = Some(val);
    //     self.is_full.notify_one();
    // }


    // pub fn remove(&self) -> i32{
    //     let mut lock = self.data.lock().unwrap();
    //     println!("remove {:?}",lock);
    //     while lock.is_none() {
    //         lock = self.is_empty.wait(lock).unwrap();
    //         self.is_empty.notify_one();
    //     }
    //    let val  = lock.take().unwrap();
    //    self.is_empty.notify_one();
    //    val
    // }

    fn insert(&self, val: i32) {
        let mut lock = self.data.lock().expect("Can't lock");
        while lock.is_some() {
            lock = self.data_cv.wait(lock).expect("Can't wait");
        }
        *lock = Some(val);
        self.data_cv.notify_one();
    }

    fn remove(&self) -> i32 {
        let mut lock = self.data.lock().expect("Can't lock");
        while lock.is_none() {
            lock = self.data_cv.wait(lock).expect("Can't wait");
        }
        let val = lock.take().unwrap();
        self.data_cv.notify_one();
        val
    }
}

fn producer(buf: &Buffer) {
    for i in 0..50 {
        buf.insert(i);
    }
}

fn consumer(buf: &Buffer) {
    for _ in 0..50 {
        println!("{}",buf.remove());
    }
}

pub fn c_cond_var() {
    let buf =  Arc::new(Buffer::default());
    let b  = Arc::clone(&buf);
    let bc = Arc::clone(&buf);
    
    let p = thread::spawn(move|| {
        producer(&b);
    });
    let c = thread::spawn(move || {
        consumer(&bc);
    });
    
    p.join().unwrap();
    c.join().unwrap();
}