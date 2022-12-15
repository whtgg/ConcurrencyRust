// use std::time;

// use async_std::task::sleep;

pub fn runtime_tokio() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("in block_on");
        rt.spawn(async {
           //  sleep(time::Duration::from_millis(1000)).await;
            println!("in spawn");
        });
    });
    rt.spawn_blocking(|| println!("spawn blocking"));
}