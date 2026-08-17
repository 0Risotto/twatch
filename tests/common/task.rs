use std::future::Future;
use std::pin::Pin;
use twatch::domain::common::spawner::TaskSpawner;

pub struct MockTaskSpawner {
    rt: tokio::runtime::Runtime,
}

impl MockTaskSpawner {
    pub fn new() -> Self {
        Self { rt: tokio::runtime::Runtime::new().unwrap() }
    }
}

impl TaskSpawner for MockTaskSpawner {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.rt.block_on(future);
    }
}
