use crate::message::{ClientMessage, ServerMessage};

pub trait BattleEndpoint<ID, const AS: usize> {
    fn send(&mut self, message: ServerMessage<ID, AS>);

    fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>>;
}

#[derive(Debug)]
pub enum ReceiveError {
    Disconnected,
}