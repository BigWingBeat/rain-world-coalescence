use std::{collections::VecDeque, io::Read, ops::Deref};

use bytes::Bytes;

/// A view into some bytes in a [`ByteQueue`]
#[derive(Debug)]
pub struct Peek<'a> {
    queue: &'a mut ByteQueue,
    bytes: Box<[u8]>,
}

impl Peek<'_> {
    /// Consume the bytes from the queue and return them
    pub fn take(self) -> Box<[u8]> {
        self.queue.discard_bytes(self.bytes.len());
        self.bytes
    }
}

impl AsRef<[u8]> for Peek<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Deref for Peek<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

/// A queue of byte slices that may be discontiguous in memory
#[derive(Debug, Default)]
pub struct ByteQueue {
    queue: VecDeque<Bytes>,
    total_bytes: usize,
}

impl ByteQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            total_bytes: 0,
        }
    }

    /// Pushes `received` to the back of the queue, if it is not empty.
    pub fn push(&mut self, received: Bytes) {
        if !received.is_empty() {
            self.total_bytes += received.len();
            self.queue.push_back(received);
        }
    }

    pub fn len(&self) -> usize {
        self.total_bytes
    }

    pub fn is_empty(&self) -> bool {
        self.total_bytes == 0
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.total_bytes = 0;
    }

    /// Returns a copy of the first byte in the queue, or None if the queue is empty
    pub fn first(&self) -> Option<u8> {
        self.queue.front().and_then(|bytes| bytes.first()).copied()
    }

    /// Returns a view into the first `amount` bytes of the queue, without immediately consuming them from the queue.
    ///
    /// If the specified amount of bytes is greater than the entire queue length, a view of the entire queue is returned.
    pub fn peek(&mut self, amount: usize) -> Peek<'_> {
        if self.is_empty() {
            return Peek {
                queue: self,
                bytes: Box::new([]),
            };
        }

        let mut amount = amount.min(self.len());
        let mut buf = Vec::with_capacity(amount);
        let mut iter = self.queue.iter();

        while amount > 0 {
            // SAFETY: `amount.min(self.len())` above gurantees that the while loop will terminate before we run out of bytes
            let next = unsafe { iter.next().unwrap_unchecked() };
            if next.len() > amount {
                // SAFETY: The range ..amount is always in bounds because we checked that it is less than the slice len above
                buf.extend_from_slice(unsafe { next.get_unchecked(..amount) });
                amount = 0;
            } else {
                buf.extend_from_slice(next);
                amount -= next.len();
            }
        }

        Peek {
            queue: self,
            bytes: buf.into_boxed_slice(),
        }
    }

    /// Removes the first `to_discard` bytes from the queue.
    pub fn discard_bytes(&mut self, mut to_discard: usize) {
        use bytes::Buf;

        if self.is_empty() {
            return;
        }

        if to_discard >= self.total_bytes {
            self.clear();
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
        if self.is_empty() {
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
        }

        self.total_bytes -= dst_len;
        Ok(dst_len)
    }
}
