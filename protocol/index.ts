// Re-export all generated types from the Rust protocol definition
export type {
  Uuid,
  Value,
  RoomId,
  SessionId,
  Credentials,
  UserProfile,
  ChatMessage,
  Participant,
  RoomEvent,
  SessionDescription,
  SessionCommand,
  SessionMessage,
  ChatRoomCommand,
} from './generated.ts';