use signaler_protocol as protocol;

#[xactor::message]
pub struct Foo;

#[xactor::message]
#[derive(Debug)]
pub struct Command(protocol::SessionCommand);

impl From<protocol::SessionCommand> for Command {
    fn from(sc: protocol::SessionCommand) -> Self {
        Self(sc)
    }
}
