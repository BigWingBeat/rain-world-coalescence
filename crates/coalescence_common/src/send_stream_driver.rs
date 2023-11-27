use bevy::tasks::{block_on, IoTaskPool, Task};
use bytes::Bytes;
use futures_lite::future::poll_once;
use quinn::{SendStream, WriteError};

#[derive(Debug)]
enum SendStreamState {
    Idle(SendStream),
    Writing(Task<(Result<(), WriteError>, SendStream)>),
}

#[derive(Debug)]
pub struct SendStreamDriver {
    send_queue: Vec<Bytes>,
    state: SendStreamState,
}

impl SendStreamDriver {
    pub fn new(send: SendStream) -> Self {
        Self {
            send_queue: Vec::new(),
            state: SendStreamState::Idle(send),
        }
    }

    pub fn queue_chunk(&mut self, chunk: Bytes) {
        self.send_queue.push(chunk);
    }

    pub fn queue_chunks(&mut self, chunks: impl IntoIterator<Item = Bytes>) {
        self.send_queue.extend(chunks);
    }

    pub fn drive(&mut self) -> Result<(), WriteError> {
        if let SendStreamState::Idle(send) = &self.state {
            if self.send_queue.is_empty() {
                return Ok(());
            }

            let mut to_send = std::mem::take(&mut self.send_queue);

            // Get the taskpool before the unsafe block so if it panics nothing bad happens
            let taskpool = IoTaskPool::get();

            // SAFETY:
            // This behaves like `std::mem::replace`. We ptr::read from `self.state`,
            // then ptr::write to it immediately after, so the old value is not duplicated or dropped
            unsafe {
                let mut send = std::ptr::read(send);
                let task = taskpool
                    .spawn(async move { (send.write_all_chunks(&mut to_send).await, send) });
                std::ptr::write(&mut self.state, SendStreamState::Writing(task));
            }
        }

        let SendStreamState::Writing(task) = &mut self.state else {
            unreachable!();
        };

        block_on(poll_once(task)).map_or(Ok(()), |(result, send)| {
            self.state = SendStreamState::Idle(send);
            result
        })
    }
}
