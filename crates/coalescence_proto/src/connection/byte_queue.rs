use std::{collections::VecDeque, io::Read};

use bytes::Bytes;

/// A queue of byte slices that may be discontiguous in memory
#[derive(Debug)]
pub struct ByteQueue {
    queue: VecDeque<Bytes>,
    total_bytes: usize,
}

impl ByteQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            total_bytes: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            total_bytes: 0,
        }
    }

    pub fn push(&mut self, received: Bytes) {
        if !received.is_empty() {
            self.total_bytes += received.len();
            self.queue.push_back(received);
        }
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub fn discard_bytes(&mut self, mut to_discard: usize) {
        use bytes::Buf;

        if self.queue.is_empty() {
            return;
        }

        if to_discard >= self.total_bytes {
            self.queue.clear();
            self.total_bytes = 0;
            return;
        }

        self.total_bytes -= to_discard;

        while to_discard > 0 {
            // SAFETY: We checked if the queue is empty above, so .front_mut() here will always return an element
            let front = unsafe { self.queue.front_mut().unwrap_unchecked() };

            let front_len = front.len();
            if front_len > to_discard {
                front.advance(to_discard);
                return;
            } else {
                self.queue.pop_front();
                to_discard -= front_len;
            }
        }
    }
}

impl Read for ByteQueue {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.queue.is_empty() {
            return Ok(0);
        }

        // SAFETY: We checked if the queue is empty above, so .front_mut() here will always return an element
        let front = unsafe { self.queue.front_mut().unwrap_unchecked() };

        let src_len = front.len();
        let dst_len = src_len.min(buf.len());

        // SAFETY: The range ..dst_len is always in bounds because it is calculated from .min(buf.len()) above
        let dst = unsafe { buf.get_unchecked_mut(..dst_len) };

        if src_len > dst_len {
            dst.copy_from_slice(&front.split_to(dst_len));
        } else {
            dst.copy_from_slice(front);
            self.queue.pop_front();
        };

        self.total_bytes -= dst_len;
        Ok(dst_len)
    }
}
