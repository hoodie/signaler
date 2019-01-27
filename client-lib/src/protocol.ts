export type Command =
    | { type: "join", room: string }
    | { type: "message", message: { content: string }, room: string }
    | { type: "listRooms" }
    | { type: "listMyRooms" }
    | { type: "shutDown" };


export type Message = {
    type: 'welcome'
    session: { uuid: string }
}

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