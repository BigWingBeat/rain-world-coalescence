use bevy::tasks::{block_on, IoTaskPool, Task};
use futures_lite::future::poll_once;
use quinn::{Chunk, ReadError, RecvStream};

#[derive(Debug)]
enum ReceiveStreamState {
    Idle(RecvStream),
    Receiving(Task<(Result<Option<Chunk>, ReadError>, RecvStream)>),
}

#[derive(Debug)]
pub struct ReceiveStreamDriver {
    state: ReceiveStreamState,
}

impl ReceiveStreamDriver {
    pub fn new(receive: RecvStream) -> Self {
        Self {
            state: ReceiveStreamState::Idle(receive),
        }
    }

    pub fn try_receive(
        &mut self,
        max_length: usize,
        ordered: bool,
    ) -> Option<Result<Option<Chunk>, ReadError>> {
        if let ReceiveStreamState::Idle(receive) = &self.state {
            // Get the taskpool before the unsafe block so if it panics nothing bad happens
            let taskpool = IoTaskPool::get();

            // SAFETY:
            // This behaves like `std::mem::replace`. We ptr::read from `self.state`,
            // then ptr::write to it immediately after, so the old value is not duplicated or dropped
            unsafe {
                let mut receive = std::ptr::read(receive);
                let task = taskpool
                    .spawn(async move { (receive.read_chunk(max_length, ordered).await, receive) });
                std::ptr::write(&mut self.state, ReceiveStreamState::Receiving(task));
            }
        }

        let ReceiveStreamState::Receiving(task) = &mut self.state else {
            unreachable!();
        };

        block_on(poll_once(task)).map(|(result, receive)| {
            self.state = ReceiveStreamState::Idle(receive);
            result
        })
    }
}
