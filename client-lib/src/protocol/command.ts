export interface Authenticate { type: 'authenticate', credentials: Credentials }

export type Credentials = 
    | { type: 'adHoc', username: string; }
    | { type: 'usernamePassword', username: string; password: string; }

// global
export interface Join { type: 'join', room: string }
export interface Leave { type: 'leave', room: string }
export interface ListRooms { type: 'listRooms' }
export interface Shutdown { type: 'shutDown' };

// chat Room
export interface Message { type: 'message', content: string }
export interface ListMyRooms { type: 'listMyRooms' }
export interface ChatRoomCommand {type: 'chatRoom', command: Message | ListMyRooms, room: string }

export type Command =
    | Authenticate
    | Join
    | Leave
    | ChatRoomCommand
    | ListRooms
    | ListMyRooms
    | Shutdown;
