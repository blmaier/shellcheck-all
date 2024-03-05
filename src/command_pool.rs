use tokio::process::Command;
//use std::collections::VecDeque;
use tokio::task::JoinSet;
use std::process::Output;
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct CommandPool {
    tasks: JoinSet<Result<Output, std::io::Error>>,
    running: Arc<Semaphore>,
}

impl CommandPool {
    pub fn new(num_threads: usize) -> Self {
        CommandPool{
            tasks: JoinSet::new(),
            running: Arc::new(Semaphore::new(num_threads)),
        }
    }

    pub fn spawn(&mut self, mut command: Command) {
        let semaphore = self.running.clone();
        self.tasks.spawn(
            async move {
                let permit = semaphore.acquire_owned().await.unwrap();
                let result = command.output().await;
                drop(permit);
                result
            }
        );
    }

    pub async fn next(&mut self) -> Option<Result<Output, std::io::Error>> {
        self.tasks.join_next().await.map(|r| r.unwrap())
    }
}
