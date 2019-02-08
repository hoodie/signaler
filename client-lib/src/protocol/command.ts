export interface Authenticate { type: 'authenticate', credentials: UsernamePassword }

export interface UsernamePassword {
    username: string;
    password: string;
}

export interface Join { type: 'join', room: string }
export interface Message { type: 'message', message: string , room: string }
export interface ListRooms { type: 'listRooms' }
export interface ListMyRooms { type: 'listMyRooms' }
export interface Shutdown { type: 'shutDown' };


export type Command =
    | Authenticate
    | Join
    | Message
    | ListRooms
    | ListMyRooms
    | Shutdown;
