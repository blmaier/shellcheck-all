use tokio::process::Command;
//use std::collections::VecDeque;
use tokio::task::JoinSet;
use std::process::Output;
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct CommandPool<T> {
    tasks: JoinSet<(T, Result<Output, std::io::Error>)>,
    running: Arc<Semaphore>,
}

impl<T> CommandPool<T>
where
    T: Send + 'static
{
    pub fn new(num_threads: usize) -> Self {
        CommandPool{
            tasks: JoinSet::new(),
            running: Arc::new(Semaphore::new(num_threads)),
        }
    }

    pub fn spawn(&mut self, mut command: Command, data: T) {
        let semaphore = self.running.clone();
        self.tasks.spawn(
            async move {
                let rdata = data;
                let permit = semaphore.acquire_owned().await.unwrap();
                let result = command.output().await;
                drop(permit);
                (rdata, result)
            }
        );
    }

    pub async fn next(&mut self) -> Option<(T, Result<Output, std::io::Error>)> {
        self.tasks.join_next().await.map(|r| r.unwrap())
    }
}
