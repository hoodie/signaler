export interface CommandAuthenticate { type: 'authenticate', credentials: UsernamePassword }
export interface CommandJoin { type: 'join', room: string }
export interface CommandMessage { type: 'message', message: string , room: string }
export interface CommandListRooms { type: 'listRooms' }
export interface CommandListMyRooms { type: 'listMyRooms' }
export interface CommandShutdown { type: 'shutDown' };

export interface UsernamePassword {
    username: string;
    password: string;
}

export type Command =
    | CommandAuthenticate
    | CommandJoin
    | CommandMessage
    | CommandListRooms
    | CommandListMyRooms
    | CommandShutdown;

export interface MessageWelcome { type: 'welcome', session: SessionDescription }
export interface MessageRoomList { type: 'roomList', rooms: string[] }
export interface MessageMyRoomList { type: 'myRoomList', rooms: string[] }
export interface MessageMessage { type: 'message', message: ChatMessage, room: string }
export interface MessageAny { type: 'any', payload: any }
export interface MessageOk { type: 'ok' }

export const isWelcomeMessage = (msg: any): msg is MessageWelcome =>
    typeof msg === 'object' && msg.type === 'welcome' && isSessionDescription(msg.session);

export interface SessionDescription {
    session_id: string;
}

export const isSessionDescription = (d: any) => typeof d === 'object' && typeof d.session_id === 'string';

export interface ChatMessage {
    content: string;
    sender: string;
    received: Date;
}

export type Message =
    | MessageWelcome
    | MessageRoomList
    | MessageMyRoomList
    | MessageMessage
    | MessageAny
    | MessageOk;
