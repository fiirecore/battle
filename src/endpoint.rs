use crate::message::{ClientMessage, ServerMessage};

/// Represents an endpoint for the battle host.
pub trait BattleEndpoint<A, B> {
    fn send(&self, message: A) -> Result<(), ConnectionError>;

    fn receive(&self) -> Result<Option<B>, ConnectionError>;
}

#[derive(Debug)]
pub enum ConnectionError {
    Disconnected,
}

#[cfg(feature = "mpsc_endpoint")]
pub use mpsc::*;

#[cfg(feature = "mpsc_endpoint")]
mod mpsc {

    use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError, TrySendError};

    use crate::message::{ClientMessage, ServerMessage};

    use super::{BattleEndpoint, ConnectionError};

    pub fn create<A, B>() -> (MpscConnection<A, B>, MpscConnection<B, A>) {
        let (sender, serv_receiver) = unbounded();
        let (serv_sender, receiver) = unbounded();

        (
            MpscConnection { sender, receiver },
            MpscConnection {
                receiver: serv_receiver,
                sender: serv_sender,
            },
        )
    }

    #[derive(Clone)]
    pub struct MpscConnection<A, B> {
        pub sender: Sender<A>,
        pub receiver: Receiver<B>,
    }

    pub type MpscClient<ID, T> = MpscConnection<ClientMessage<ID>, ServerMessage<ID, T>>;
    pub type MpscEndpoint<ID, T> = MpscConnection<ServerMessage<ID, T>, ClientMessage<ID>>;

    impl<A, B> BattleEndpoint<A, B> for MpscConnection<A, B> {
        fn send(&self, message: A) -> Result<(), ConnectionError> {
            match self.sender.try_send(message) {
                Ok(()) => Ok(()),
                Err(TrySendError::Full(..)) => unreachable!(),
                Err(TrySendError::Disconnected(..)) => Err(ConnectionError::Disconnected),
            }
        }

        fn receive(&self) -> Result<Option<B>, ConnectionError> {
            match self.receiver.try_recv() {
                Ok(m) => Ok(Some(m)),
                Err(TryRecvError::Empty) => Ok(None),
                Err(TryRecvError::Disconnected) => Err(ConnectionError::Disconnected),
            }
        }
    }
}
