use std::{sync::{Arc, Mutex}, task::{Waker, Poll}, time::Duration, thread};
use std::future::Future as stdFuture;

use futures_lite::{future, io, AsyncReadExt};

pub fn f_block() {
    future::block_on(async {
        let input:&[u8] = b"hello world";
        let mut reader  = io::BufReader::new(input);
        let mut s = String::new();
        reader.read_to_string(&mut s).await.unwrap();
        println!("{}",s);
    });
}


/// 自定义Future
pub struct TimerFuture {
    shared_state: Arc<Mutex<ShareState>>,
}

struct ShareState {
    is_completed: bool,
    waker: Option<Waker>,
}

impl TimerFuture {
    pub fn new(duration:Duration) -> Self{
        let shared_state = Arc::new(Mutex::new(ShareState{
            is_completed: false,
            waker:None,
        }));
        
        let shared_state_clone = Arc::clone(&shared_state);
        
        thread::spawn(move || {
            let mut lock = shared_state_clone.lock().unwrap();
            thread::sleep(duration);
            lock.is_completed = true;
            if let Some(waker) = lock.waker.take() {
                waker.wake();
            }
        });
        
        TimerFuture {
            shared_state,
        }
    }
}

impl stdFuture for TimerFuture {
    type Output = ();
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut lock = self.shared_state.lock().unwrap();
        
        if lock.is_completed {
            println!("TimerFuture completed");
            Poll::Ready(())
        } else {
            lock.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}


pub fn f_timer_async() {
    let tf = TimerFuture::new(Duration::from_millis(1000));
    future::block_on(tf);
}



