use std::process::Stdio;

use tokio::{process::Command, io::{BufReader, AsyncBufReadExt}, sync::{oneshot, broadcast, watch}, time::{Duration, interval}};

/// tokio Command
pub fn t_process() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mut child = Command::new("echo")
            .arg("hello")
            .arg("world")
            .spawn()
            .expect("faild to spawn");
        
        let status = child.wait().await.unwrap();
        println!("exit with status is {}",status);
    });
}

/// tokio Command 读取文件并输出
pub fn t_process_stdout() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mut cmd = Command::new("cat");
        cmd.arg("Cargo.toml");
        cmd.stdout(Stdio::piped());
        let mut  child = cmd.spawn().expect("spawn failed");
        
        let out = child.stdout
            .take()
            .expect("stdout fail");
        
        let mut reader = BufReader::new(out).lines();
        
        tokio::spawn(async move{
            let status = child
                .wait()
                .await
                .expect("wait status");
            println!("child status is {}",status);
        });
        
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("line is {}",line);
        }
    });
}

/// 单值channel
pub fn t_oneshot() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (tx,mut rx) =  oneshot::channel();
    rt.block_on(async  {
        rt.spawn(async {
            if let Err(e) = tx.send(1) {
                println!("{}",e);
            }
        });
        // rt.spawn(async {
        //    match rx.await {
        //         Ok(data) => {
        //             println!(" rx data is {}",data);
        //         },
        //         _ => {},
        //    }
        // });

        // match rx.await {
        //     Ok(data) => {
        //         println!(" rx data is {}",data);
        //     },
        //     _ => {},
        // }
        let mut interval = interval(Duration::from_millis(100));
        tokio::select! {
            _ = interval.tick() => println!("Another time"),
            msg = &mut rx => {
                println!("receive msg {:?}",msg);
            },
        }
        
        // sleep(time::Duration::from_secs(1)).await;
    });
}


/// block_on 调用外部异步函数
pub fn t_async_fn() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    async fn compution() -> String {
        "we are world".to_string()
    }
    rt.block_on(async {
       let handle = tokio::spawn(async {
            compution().await
       });
       let res =  handle.await.unwrap();
       println!("res is {}",res);
    });
}

/// channel 广播
pub fn t_channel_brocast() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let (tx,mut rx) = broadcast::channel::<String>(10);
        let mut rx_b = tx.subscribe();// 订阅
        tokio::spawn(async move {
            tx.send("rust basic".into()).unwrap();
            tx.send("rust advanced".into()).unwrap();
        });
        println!("rx receive {}",rx.recv().await.unwrap());
        println!("rx receive {}",rx.recv().await.unwrap());
        println!("rx_b receive {}",rx_b.recv().await.unwrap());
    });
}

/// 监听某个channel流
pub fn t_channel_watch() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let (tx,rx) = watch::channel("rust need basic");
        let mut rx_w = tx.subscribe();
        tokio::spawn(async move {
            tx.send("rust need advanced").unwrap();
        });
        println!("rx borrow {:?}",rx.borrow());
        println!("rx_w borrow {:?}",rx_w.borrow());
        println!("rx_w change {:?}",rx_w.changed().await.unwrap());
        // println!("rx_w change {:?}",rx_w.changed().await.unwrap());
    });
}
