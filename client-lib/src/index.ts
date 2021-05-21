import { Signal, ReadableSignal, Listener } from 'micro-signals';

import { Command, UserProfile, RoomParticipants } from './protocol';
import { serverEvent, ServerEvent } from './protocol';
import { SessionDescription, isWelcomeEvent, ChatMessage } from './protocol/index'; // weird webpack bug

export { ChatMessage, SessionDescription };


const timeout = (time: number): Promise<never> => new Promise((_, reject) => setTimeout(reject, time));

export class Session {
    private connection?: WebSocket;

    public sessionId?: string;

    private _onWelcome = new Signal<SessionDescription>();
    public readonly onWelcome = this._onWelcome.readOnly();
    public readonly onAuthenticated = new Signal<void>();
    public readonly onProfile = new Signal<UserProfile>();

    public readonly onReceive = new Signal<any>();
    public readonly onRoomList = new Signal<string[]>();
    public readonly onMyRoomList = new Signal<string[]>();
    public readonly onRoomParticipants = new Signal<RoomParticipants>();
    public readonly onMessage = new Signal<{room: string, message: ChatMessage}>();
    public readonly onError = new Signal<string>();

    public readonly onConnectionClose = new Signal<CloseEvent>();
    public readonly onConnectionError = new Signal<Event>();

    constructor(private url: string) {
        this.onReceive.add(m => console.debug('⬅️ received', m));
    }

    public connect(): Promise<SessionDescription> {
        if (this.connection) return Promise.reject(new Error("already connected"));

        console.debug({isWelcomeEvent});
        const connected = this.onReceive.filter(isWelcomeEvent).map(welcome => welcome.session).promisify();

        this.connection = new WebSocket(this.url);

        this.connection.onmessage = (rawMsg: MessageEvent) => {
            console.debug({rawMsg})
            try {
                this.handle(JSON.parse(rawMsg.data));
            } catch (error) {
                console.error('can\'t parse', rawMsg.data, error);
            }
        };

        this.connection.onclose = (ev: CloseEvent) => this.onConnectionClose.dispatch(ev);
        this.connection.onerror = (ev: Event) => this.onConnectionError.dispatch(ev);

        return connected;
    }

    public disconnect() {
        if (!this.connection) return;

        this.connection.close();
        this.connection = undefined;
    }

    private handle(msg: ServerEvent) {
        this.onReceive.dispatch(msg);
        switch (msg.type) {
            case 'authenticated': return this.onAuthenticated.dispatch();
            case 'profile': return this.onProfile.dispatch(msg.profile);
            case 'welcome': return this._onWelcome.dispatch(msg.session);
            case 'roomList': return this.onRoomList.dispatch(msg.rooms);
            case 'myRoomList': return this.onMyRoomList.dispatch(msg.rooms);
            case 'roomParticipants': return this.onRoomParticipants.dispatch(msg);
            case 'message': {
                console.info('chatmessage received', msg);
                this.onMessage.dispatch({
                    message: {
                        ...msg.message,
                        received: new Date(),
                        sent: new Date(msg.message.sent),
                    },
                    room: msg.room,
                });
                return;
            }
            case 'any': return console.debug(msg.payload);
            case 'error': return this.onError.dispatch(msg.message);
            default: return console.warn('unhandled message', msg);
        }
    }

    public sendCommand(cmd: Command) {
        console.debug('➡️ sending', cmd)
        if (!this.connection) {
            throw new Error("connection already closed, can't send")
        }
        this.connection?.send(JSON.stringify(cmd));
    }

    public adHoc(username: string): Promise<void> {
        const authenticated = this.onAuthenticated.promisify();
        const type = 'adHoc';
        this.sendCommand({ type: 'authenticate', credentials: {type, username } });
        return Promise.race([authenticated, timeout(1000)]);
    }

    public authenticate(username: string, password: string): Promise<void> {
        const authenticated = this.onAuthenticated.promisify();
        const type = 'usernamePassword';
        this.sendCommand({ type: 'authenticate', credentials: {type, username, password } });
        return Promise.race([authenticated, timeout(1000)]);
    }

    public join(room: string): RoomHandle {
        this.sendCommand({ type: 'join', room });
        return this.createRoomHandle(room);
    }

    public leave(room: string) {
        this.sendCommand({ type: 'leave', room });
    }

    public listRooms() {
        this.sendCommand({ type: 'listRooms' })
    }

    public listMyRooms() {
        this.sendCommand({ type: 'listMyRooms' })
    }

    public sendMessage(message: string, room: string) {
        this.sendCommand({ type: 'chatRoom', command: { type: 'message',  content: message } , room });
    }

    public createRoomHandle(room: string): RoomHandle {
        return createRoomHandle(this, room);
    }

}

interface RoomHandle {
    readonly onMessage: ReadableSignal<ChatMessage>;
    send(messages: string): void;
}

function createRoomHandle(session: Session, room: string): RoomHandle {
    const onMessage = session.onMessage.filter(msg => msg.room === room).map(msg => msg.message);
    const send = (msg: string) => session.sendMessage(msg, room);

    return {
        onMessage, send
    }
}