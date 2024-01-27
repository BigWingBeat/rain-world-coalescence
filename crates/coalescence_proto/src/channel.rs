/// The Reliable-Ordered channel: Packets are guaranteed to arrive in the same order they are sent
#[derive(Debug)]
pub enum Ordered {}

/// The Reliable-Unordered channel: Packets are guaranteed to arrive, but possibly in a different order than they were sent
#[derive(Debug)]
pub enum Unordered {}

/// The Unreliable-Unordered channel: Packets may not arrive, or may arrive in a different order than they were sent
#[derive(Debug)]
pub enum Unreliable {}

/// Different ways that packets can be transmitted over the network
pub trait Channel: sealed::Sealed + 'static {}

impl Channel for Ordered {}
impl Channel for Unordered {}
impl Channel for Unreliable {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Ordered {}
    impl Sealed for super::Unordered {}
    impl Sealed for super::Unreliable {}
}
