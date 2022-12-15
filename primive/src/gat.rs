

// use async_trait::*;
use std::future::Future;
use async_std::io::WriteExt;
use bytes::Bytes;
// #[async_trait]
// pub trait  KvIterator {
//     async fn next(&self) -> Option<(&[u8],&[u8])>;
// }

/// 使用 GAT 写 async trait
/// 参考 https://www.sobyte.net/post/2022-04/rust-gat-async-trait/
/// 参考 https://zhuanlan.zhihu.com/p/463367405
pub trait KvIterator {
    type NextFuture<'a>: Future<Output = Option<(&'a [u8],&'a [u8])>>
    where
        Self: 'a;
    
    fn next(&mut self) -> Self::NextFuture<'_>;
}

pub struct TestIterator {
    idx: usize,
    to_idx: usize,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl TestIterator {
    pub fn new(from_idx: usize, to_idx: usize) -> Self {
        Self {
            idx: from_idx,
            to_idx,
            key: Vec::new(),
            value: Vec::new(),
        }
    }
}

impl KvIterator for TestIterator {
    type NextFuture<'a> = impl Future<Output = Option<(&'a [u8], &'a [u8])>>;
    
    fn next(&mut self) -> Self::NextFuture<'_> {
        async move { 
            if self.idx >= self.to_idx {
                return None;
            }

            // Zero-allocation key value manipulation

            self.key.clear();
            write!(&mut self.key, "key_{:05}", self.idx).await.unwrap();

            self.value.clear();
            write!(&mut self.value, "value_{:05}", self.idx).await.unwrap();

            self.idx += 1;
            Some((&self.key[..], &self.value[..]))
        }
    }
}

pub fn kviterator_test() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut iter = TestIterator::new(0, 10);
        while let Some((key, value)) = iter.next().await {
            println!(
                "{:?} {:?}",
                Bytes::copy_from_slice(key),
                Bytes::copy_from_slice(value)
            );
        }
    });
}