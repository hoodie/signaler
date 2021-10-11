use signaler_protocol::{
    ChatMessage, ChatRoomCommand, Credentials, Participant, RoomEvent, RoomId, SessionCommand, SessionDescription,
    SessionId, SessionMessage, UserProfile,
};
use typescript_definitions::{TypeScriptify, TypeScriptifyTrait};
fn main() {
    println!("{}", RoomId::type_script_ify());
    // println!("{}", SessionId::type_script_ify());
    println!("export type SessionId = String;");
    println!("{}", Credentials::type_script_ify());
    println!("{}", UserProfile::type_script_ify());
    println!("{}", ChatMessage::type_script_ify());
    println!("{}", Participant::type_script_ify());
    println!("{}", RoomEvent::type_script_ify());
    println!("{}", SessionDescription::type_script_ify());
    println!("{}", SessionCommand::type_script_ify());
    println!("{}", SessionMessage::type_script_ify());
    println!("{}", ChatRoomCommand::type_script_ify());
}
