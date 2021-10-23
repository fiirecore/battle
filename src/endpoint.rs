use crate::message::{ClientMessage, ServerMessage};

pub trait BattleEndpoint<ID, const AS: usize> {
    fn send(&mut self, message: ServerMessage<ID, AS>);

    fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>>;
}

#[derive(Debug)]
pub enum ReceiveError {
    Disconnected,
}

#[cfg(feature = "default_endpoint")]
pub use endpoints::*;

#[cfg(feature = "default_endpoint")]
mod endpoints {

    use crossbeam_channel::{Receiver, Sender, TryRecvError, unbounded};

    use crate::message::{ClientMessage, ServerMessage};

    use super::{BattleEndpoint, ReceiveError};

    pub fn create<ID, const AS: usize>() -> (MpscClient<ID, AS>, MpscEndpoint<ID, AS>) {
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
    pub struct MpscClient<ID, const AS: usize> {
        pub sender: Sender<ClientMessage<ID>>,
        pub receiver: Receiver<ServerMessage<ID, AS>>,
    }

    #[derive(Clone)]
    pub struct MpscEndpoint<ID, const AS: usize> {
        pub receiver: Receiver<ClientMessage<ID>>,
        pub sender: Sender<ServerMessage<ID, AS>>,
    }

    impl<ID, const AS: usize> MpscClient<ID, AS> {
        pub fn send(&self, message: ClientMessage<ID>) {
            if let Err(err) = self.sender.try_send(message) {
                log::error!("AI cannot send client message with error {}", err);
            }
        }
    }
    
    impl<ID, const AS: usize> BattleEndpoint<ID, AS> for MpscEndpoint<ID, AS> {
        fn send(&mut self, message: ServerMessage<ID, AS>) {
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
