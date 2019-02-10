import { SessionDescription, ChatMessage } from ".";

export interface Welcome { type: 'welcome', session: SessionDescription }
export interface RoomList { type: 'roomList', rooms: string[] }
export interface MyRoomList { type: 'myRoomList', rooms: string[] }
export interface Message { type: 'message', message: ChatMessage, room: string }
export interface Any { type: 'any', payload: any }
export interface Ok { type: 'ok' }
export interface Error { type: 'error', message: string }

export type ServerEvent =
    | Welcome
    | RoomList
    | MyRoomList
    | Message
    | Any
    | Ok
    | Error;