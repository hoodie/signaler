export type RoomId = string;
export type SessionId = String;
export type Credentials = 
 | { type: "usernamePassword"; username: string; password: string } 
 | { type: "adHoc"; username: string };
export type UserProfile = { fullName: string };
// Actual chat Message
// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
export type ChatMessage = { content: string; sender: SessionId; sent: string; uuid: Uuid };
// SessionId and Full Name
// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
export type Participant = { fullName: string; sessionId: SessionId };
export type RoomEvent = 
 | { participantJoined: { name: string } } 
 | { participantLeft: { name: string } };
export type SessionDescription = { sessionId: SessionId };
// Command sent to the server
export type SessionCommand = 
 | { type: "join"; room: RoomId } 
 | { type: "chatRoom"; room: RoomId; command: ChatRoomCommand } 
 | { type: "listRooms" } 
 | { type: "listMyRooms" } 
 | { type: "shutDown" } 
 | { type: "authenticate"; credentials: Credentials };
// Message received from the server
export type SessionMessage = 
 | { type: "welcome"; session: SessionDescription } 
 | { type: "authenticated" } 
 | { type: "profile"; profile: UserProfile } 
 | { type: "roomList"; rooms: string [] } 
 | { type: "myRoomList"; rooms: string [] } 
 | { type: "roomParticipants"; room: RoomId; participants: Participant [] } 
 | { type: "roomEvent"; room: RoomId; event: RoomEvent } 
 | { type: "message"; message: ChatMessage; room: RoomId } 
 | { type: "any"; payload: Value } 
 | { type: "error"; message: string };
// Command sent to the server
export type ChatRoomCommand = 
 | { type: "leave" } 
 | { type: "message"; content: string } 
 | { type: "listParticipants" };
