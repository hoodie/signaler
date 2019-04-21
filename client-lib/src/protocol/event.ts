import { SessionDescription, ChatMessage, UserProfile } from ".";

export interface Welcome { type: 'welcome', session: SessionDescription }
export interface Authenticated { type: 'authenticated' }
export interface Profile { type: 'profile', profile: UserProfile }
export interface RoomList { type: 'roomList', rooms: string[] }
export interface MyRoomList { type: 'myRoomList', rooms: string[] }
export interface Message { type: 'message', message: ChatMessage, room: string }
export interface Any { type: 'any', payload: any }
export interface Ok { type: 'ok' }
export interface Error { type: 'error', message: string }

export type ServerEvent =
    | Authenticated
    | Profile
    | Welcome
    | RoomList
    | MyRoomList
    | Message
    | Any
    | Ok
    | Error;