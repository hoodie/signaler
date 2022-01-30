use signaler_protocol as protocol;
use xactor::message;

#[message]
#[derive(Debug)]
pub struct Command(protocol::SessionCommand);

impl From<protocol::SessionCommand> for Command {
    fn from(sc: protocol::SessionCommand) -> Self {
        Self(sc)
    }
}

#[message]
#[derive(Clone, Copy, Debug)]
pub struct Gc;
