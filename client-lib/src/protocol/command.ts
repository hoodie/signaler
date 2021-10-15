export interface Authenticate { type: 'authenticate', credentials: Credentials }

export type Credentials = 
    | { type: 'adHoc', username: string; }
    | { type: 'usernamePassword', username: string; password: string; }

// global
export interface Join { type: 'join', room: string }
export interface Reconnect { type: 'reconnect', sessionId: string }
export interface ListRooms { type: 'listRooms' }
export interface Shutdown { type: 'shutDown' };

// chat Room
export interface Message { type: 'message', content: string }
export interface ListMyRooms { type: 'listMyRooms' }
export interface Leave { type: 'leave' }

export type ChatRoomCommand = 
    | Message
    | ListMyRooms
    | Leave;

export interface ChatRoom {
    type: 'chatRoom',
    room: string,
    command: ChatRoomCommand,
}

export type Command =
    | Authenticate
    | Join
    | Reconnect
    | ChatRoom
    | ListRooms
    | ListMyRooms
    | Shutdown;
