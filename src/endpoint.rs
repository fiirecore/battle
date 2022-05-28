use crate::message::{ClientMessage, ServerMessage};

/// Represents a client endpoint for the battle host.
pub trait BattleEndpoint<ID, T> {
    fn send(&mut self, message: ServerMessage<ID, T>);

    fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>>;
}

#[derive(Debug)]
pub enum ReceiveError {
    Disconnected,
}

#[cfg(feature = "mpsc_endpoint")]
pub use mpsc::*;

#[cfg(feature = "mpsc_endpoint")]
mod mpsc {

    use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};

    use crate::message::{ClientMessage, ServerMessage};

    use super::{BattleEndpoint, ReceiveError};

    pub fn create<ID, T>() -> (MpscClient<ID, T>, MpscEndpoint<ID, T>) {
        let (serv_sender, receiver) = unbounded();
        let (sender, serv_receiver) = unbounded();

        (
            MpscClient { sender, receiver },
            MpscEndpoint {
                receiver: serv_receiver,
                sender: serv_sender,
            },
        )
    }

    #[derive(Clone)]
    pub struct MpscClient<ID, T> {
        pub sender: Sender<ClientMessage<ID>>,
        pub receiver: Receiver<ServerMessage<ID, T>>,
    }

    #[derive(Clone)]
    pub struct MpscEndpoint<ID, T> {
        pub receiver: Receiver<ClientMessage<ID>>,
        pub sender: Sender<ServerMessage<ID, T>>,
    }

    impl<ID, T> MpscClient<ID, T> {
        pub fn send(&self, message: ClientMessage<ID>) {
            if let Err(err) = self.sender.try_send(message) {
                log::error!("AI cannot send client message with error {}", err);
            }
        }
    }

    impl<ID, T> BattleEndpoint<ID, T> for MpscEndpoint<ID, T> {
        fn send(&mut self, message: ServerMessage<ID, T>) {
            if let Err(err) = self.sender.try_send(message) {
                log::error!("Cannot send server message to AI with error {}", err);
            }
        }

        fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>> {
            match self.receiver.try_recv() {
                Ok(m) => Ok(m),
                Err(err) => Err(match err {
                    TryRecvError::Empty => None,
                    TryRecvError::Disconnected => Some(ReceiveError::Disconnected),
                }),
            }
        }
    }
}
