use signaler_protocol as protocol;
use hannibal::message;

/// TODO: this is probably unnecessary
#[message]
#[derive(Debug)]
pub struct Command(pub protocol::SessionCommand);

impl From<protocol::SessionCommand> for Command {
    fn from(sc: protocol::SessionCommand) -> Self {
        Self(sc)
    }
}

#[message]
#[derive(Clone, Copy, Debug)]
pub struct Gc;
