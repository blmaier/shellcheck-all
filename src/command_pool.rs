
use tokio::process::Command;
use std::collections::VecDeque;
use tokio::task::JoinSet;
use std::process::Output;

pub struct CommandPoolBuilder {
    commands: VecDeque<Command>,
}

pub struct CommandPool {
    num_threads: usize,
    tasks: JoinSet<Result<Output, std::io::Error>>,
    commands: VecDeque<Command>,
}

impl CommandPoolBuilder {
    pub fn new() -> Self {
        CommandPoolBuilder {
            commands: VecDeque::new(),
        }
    }

    pub fn command(&mut self, command: Command) -> &mut Self {
        self.commands.push_back(command);
        self
    }

    pub fn build(self, num_threads: usize) -> CommandPool {
        CommandPool {
            num_threads,
            tasks: JoinSet::new(),
            commands: self.commands,
        }
    }
}

impl CommandPool {
    pub async fn next(&mut self) -> Option<Result<Output, std::io::Error>> {
        while self.tasks.len() < self.num_threads {
            if let Some(mut command) = self.commands.pop_front() {
                self.tasks.spawn(command.output());
            } else {
                break;
            }
        }
        self.tasks.join_next().await.map(|r| r.unwrap())
    }
}
