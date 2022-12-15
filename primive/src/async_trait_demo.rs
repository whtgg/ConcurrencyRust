use async_trait::*;

#[async_trait]
trait AsyncTrait {
    async fn get_string(&self) -> String;
}

#[async_trait]
impl AsyncTrait for i32 {
   async fn get_string(&self) -> String {
        12.to_string()
   }
}

pub fn get_async_trait() {
    let rt =  tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
     let s = 12.get_string().await;
     println!("{:?}",s);
   });
}