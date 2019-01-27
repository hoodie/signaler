export interface CommandJoin { type: "join", room: string }
export interface CommandMessage { type: "message", message: { content: string }, room: string }
export interface CommandListRooms { type: "listRooms" }
export interface CommandListMyRooms { type: "listMyRooms" }
export interface CommandShutdown { type: "shutDown" };

export type Command =
    | CommandJoin
    | CommandMessage
    | CommandListRooms
    | CommandListMyRooms
    | CommandShutdown;

export interface MessageWelcome { type: 'welcome', session: { uuid: string } }
export interface MessageRoomList { type: 'roomList', rooms: string[] }
export interface MessageMyRoomList { type: 'myRoomList', rooms: string[] }
export interface MessageMessage { type: 'message', message: ChatMessage }
export interface MessageAny { type: 'any', payload: any }
export interface MessageOk { type: 'ok' }

export interface ChatMessage {
    content: string;
}

export type Message =
    | MessageWelcome
    | MessageRoomList
    | MessageMyRoomList
    | MessageMessage
    | MessageAny
    | MessageOk;

export function handleMessage(msg: Message) {
    switch (msg.type)  {
        case 'welcome': return console.log(`yay, welcome, you are ${msg.session.uuid}`); 
        default: return console.warn('unhandled', msg);
    }
}

export function sendCommand(cmd: Command): Promise<void> {

    return {then(){}} as any;
}

console.debug("protocol loaded")