import * as React from 'react';

import { Session, SessionDescription, ChatMessage } from '../../client-lib/';
import { SendForm } from './SendForm';
import { AuthenticatedView } from './AuthenticatedView';

type Fn<T, R=void> = (x:T) => R;

const RoomSelector = ({ rooms, onSelect }: { rooms: string[], onSelect: Fn<string> }) => {
    return (
        <React.Fragment>
            {rooms.map(room => (
                <button key={room} onClick={() => onSelect(room)}>{room}</button>
            ))}
        </React.Fragment>
    )
};

export interface SessionViewProps {
    session: Session;
    onDisconnect: () => void;
}

interface SessionViewState {
    authenticated: boolean,
    rooms: string[];
    myrooms: string[];
    roomToSendTo: string;
    roomToJoin: string;
    receivedMessagesByRoom: { [index: string]: ChatMessage[] },

    sessionDescription?: SessionDescription;
}

const MessageList = ({ messages, me }: { messages: ChatMessage[], me?: string }) => <div className="messageList">
    {messages.map(msg => <ChatMessageView message={msg} me={me} key={msg.received.toString()} />)}
</div>

const ChatMessageView = ({ message, me }: { message: ChatMessage, me?: string }) => {
    const senderName = me === message.sender ? "me" : message.sender;
    return (<React.Fragment>
        <span className="timestamp">{message.received.getHours()}:{message.received.getMinutes()}</span>
        <span className={`sender ${senderName}`}>{senderName}</span>
        <span className="content">{message.content}</span>
    </React.Fragment>);
};

export class SessionView extends React.Component<SessionViewProps, SessionViewState> {
    private session: Session;
    private lastMessage?: HTMLSpanElement;

    constructor(props: SessionViewProps) {
        super(props);
        this.state = {
            authenticated: false,
            rooms: [],
            myrooms: [],
            receivedMessagesByRoom: {},
            roomToJoin: "default",
            roomToSendTo: "default"
        };
        this.session = props.session;

        this.session.onWelcome.add(sessionDescription => {
            this.setState({ sessionDescription })
            this.session.sendCommand({ type: 'listRooms' });
            this.session.sendCommand({ type: 'listMyRooms' });
            this.session.join('default');
        });

        this.session.onAuthenticated.add(token => {
            this.setState({ authenticated: true });
        })

        this.session.onConnectionClose.add(event => {
            console.error(event);
            this.props.onDisconnect();
        });

        this.session.onRoomList.add(rooms => {
            console.debug("onRoomList", rooms)
            this.setState({rooms})
        });

        this.session.onMyRoomList.add(myrooms => {
            console.debug("onMyRoomList", myrooms)
            this.setState({ myrooms })
        });

        this.session.onMessage.add(({ message, room }) => {
            console.debug(room, message)
            const receivedMessagesByRoom = {...this.state.receivedMessagesByRoom };
            const oldRoomMessages = receivedMessagesByRoom[room] || [];
            receivedMessagesByRoom[room] = [...oldRoomMessages, message]
            console.debug({receivedMessagesByRoom})
            this.setState({ receivedMessagesByRoom })
            this.lastMessage!.scrollIntoView({ behavior: "smooth" });
        });

        (window as any).session = this.session;
    }

    private roomToJoin: React.ChangeEventHandler<HTMLInputElement> = ({ target: { value } }) => {
        this.setState({ roomToJoin: value })
    };

    private sendToRoom = (content: string) => {
        this.session.sendMessage(content, this.state.roomToSendTo)
    };

    // TODO: temporary
    componentDidMount = () => {
        console.debug("mounted")
        this.session.connect();
    };

    private connectionView = () => {
        if (this.state.sessionDescription) {
            return (
                <React.Fragment>
                    <aside>
                        <h4>my rooms</h4>
                        <RoomSelector rooms={this.state.myrooms} onSelect={roomToSendTo => this.setState({ roomToSendTo })} />
                    </aside>

                    <section>
                        <div className="messageBox">
                            <div>
                                <MessageList messages={this.state.receivedMessagesByRoom[this.state.roomToSendTo] || []} me={this.state.sessionDescription.session_id}></MessageList>
                                <span ref={e => this.lastMessage = !e ? undefined : e} ></span>
                            </div>
                        </div>
                        <SendForm onSend={this.sendToRoom} />
                    </section>

                    <nav>
                        <small>
                            SessionId:
                            <pre>{this.state.sessionDescription.session_id}</pre>
                        </small>

                        <AuthenticatedView authenticated={this.state.authenticated} onsubmit={(u, p) => this.session.authenticate(u, p)}>
                            <h6> join room </h6>

                            <RoomSelector rooms={this.state.rooms} onSelect={room => { this.session.join(room) }} />
                            <input
                                type="text" name="channelName" id="channelName"
                                onChange={this.roomToJoin} placeholder="createNew" />
                            <button onClick={() => this.session.join(this.state.roomToJoin)}> join </button>
                        </AuthenticatedView>

                        <hr />
                        <button onClick={this.disconnect}> disconnect </button>

                    </nav>
                </React.Fragment>
            )
        } else {
            return <button onClick={() => this.session.connect()}> connect </button>
        }
    };

    private disconnect = () => {
        this.session.disconnect();
        this.props.onDisconnect();
    }

    render() {
        return (
            <div className="sessionView">
            <header>
                <h3>Session</h3>
            </header>
                {this.connectionView()}
            </div>
        )
    }
}