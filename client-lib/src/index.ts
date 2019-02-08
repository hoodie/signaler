import { Signal } from 'micro-signals';

import { Command } from './protocol';
import { serverEvent, ServerEvent } from './protocol';
import { SessionDescription, isWelcomeEvent, ChatMessage } from './protocol';

export { ChatMessage, SessionDescription };

export class Session {
    private connection?: WebSocket;

    public sessionId?: string;

    private _onWelcome = new Signal<SessionDescription>();
    public readonly onWelcome = this._onWelcome.readOnly();

    public readonly onReceive = new Signal<any>();
    public readonly onRoomList = new Signal<string[]>();
    public readonly onMyRoomList = new Signal<string[]>();
    public readonly onMessage = new Signal<serverEvent.Message>();

    public readonly onConnectionClose = new Signal<CloseEvent>();
    public readonly onConnectionError = new Signal<Event>();

    constructor(private url: string) {
        this.onReceive.add(m => console.debug('received', m));
    }

    public connect(): Promise<SessionDescription> {
        if (this.connection) return Promise.reject();

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
            case 'welcome': return this._onWelcome.dispatch(msg.session);
            case 'roomList': return this.onRoomList.dispatch(msg.rooms);
            case 'myRoomList': return this.onMyRoomList.dispatch(msg.rooms);
            case 'message': {
                console.info('chatmessage received', msg);
                this.onMessage.dispatch({...msg, message: {...msg.message, received: new Date()}});
                return;
            }
            case 'any': return console.debug(msg.payload);
            default: return console.warn('unhandle message', msg);
        }
    }

    public sendCommand(cmd: Command) {
        console.debug('sending', cmd)
        this.connection && this.connection.send(JSON.stringify(cmd));
    }

    public authenticate(username: string, password: string) {
        this.sendCommand({ type: 'authenticate', credentials: { username, password } });
    }

    public join(room: string) {
        this.sendCommand({ type: 'join', room });
    }

    public listRooms() {
        this.sendCommand({ type: 'listRooms' })
    }

    public listMyRooms() {
        this.sendCommand({ type: 'listMyRooms' })
    }

    public sendMessage(message: string, room: string) {
        this.sendCommand({ type: 'message', message, room });
    }

}
