import { Command, Message, ChatMessage } from './protocol'

export function hiThere() {
    console.info("hi there")
}

export class Session {
    private connection: WebSocket;

    constructor(url: string) {
        this.connection = new WebSocket(url);
        this.connection.onmessage = (rawMsg: MessageEvent) => {
            try {
                this.handle(JSON.parse(rawMsg.data));
            } catch (e) {
                console.error("can't parse", rawMsg.data);
            }
        };
    }

    private handle(msg: Message) {
        switch (msg.type) {
            case 'welcome': return console.info("welcome", msg.session);
            case 'roomList': return this.handleRooms(msg.rooms);
            case 'myRoomList': return this.handleMyRooms(msg.rooms);
            case 'message': return this.handleChatMessage(msg.message);
            case 'any': return console.debug(msg.payload);
            default: return console.warn('unhandle message', msg);
        }
    }

    public handleRooms = (rooms: string[]) =>
        console.info('all possible rooms' , rooms );

    public handleMyRooms = (rooms: string[]) =>
        console.info("I'm participating in " , rooms );

    public handleChatMessage = (message: ChatMessage) =>
        console.info('❤️ received', { message })

    public sendCommand(cmd: Command) {
        this.connection.send(JSON.stringify(cmd));
    }

    public join(room: string) {
        this.sendCommand({ type: "join", room })
    }

    public listRooms() {
        this.sendCommand({ type: "listRooms" })
    }

    public listMyRooms() {
        this.sendCommand({ type: "listMyRooms" })
    }

    public sendMessage(content: string, room: string) {
        this.sendCommand({ type: "message", message: { content }, room })
    }

}
