import * as React from 'react';

import { Session, SessionDescription, ChatMessage } from '../../client-lib/';
import { SendForm } from './SendForm';
import { AuthenticatedView } from './AuthenticatedView';
import { UserProfile, Participant } from '../../client-lib/src/protocol';
import { MessageList } from './MessageViews';
import { RoomSelector } from './RoomSelector';
import { Config } from '.';

export interface SessionViewProps {
    config?: Config;
    session: Session;
    onDisconnect: () => void;
}

interface SessionViewState {
    authenticated: boolean,
    rooms: string[];
    myrooms: string[];
    participantsByRoom: {[room: string]: Array<Participant>};
    roomToSendTo: string;
    roomToJoin: string;
    receivedMessagesByRoom: { [index: string]: ChatMessage[] },

    profile?: UserProfile;
    sessionDescription?: SessionDescription;
}

export class SessionView extends React.Component<SessionViewProps, SessionViewState> {
    private session: Session;
    private lastMessage?: HTMLSpanElement;

    constructor(props: SessionViewProps) {
        super(props);
        this.state = {
            authenticated: false,
            rooms: [],
            myrooms: [],
            participantsByRoom: {},
            receivedMessagesByRoom: {},
            roomToJoin: "default",
            roomToSendTo: "default"
        };
        this.session = props.session;

        this.session.onWelcome.add(sessionDescription => {
            this.setState({ sessionDescription })
            const {username, password} = this.props.config || {};
            if (username) {
                this.login(username, password)
            }
        });

        this.session.onAuthenticated.add(() => {
            this.setState({ authenticated: true });
            this.session.join('default');
        })

        this.session.onProfile.add(profile => this.setState({profile}));

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

        this.session.onRoomParticipants.add(({room, participants}) => {
            let participantsByRoom = {...this.state.participantsByRoom, [room]: participants};
            this.setState({participantsByRoom})
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

        this.session.onError.add(alert);

        (window as any).session = this.session;
    }

    private roomToJoin: React.ChangeEventHandler<HTMLInputElement> = ({ target: { value } }) => {
        this.setState({ roomToJoin: value })
    };

    private sendToRoom = (content: string) => {
        this.session.sendMessage(content, this.state.roomToSendTo)
    };

    private login = (username: string, password?: string | null) => {
        if (password) {
            this.session.authenticate(username, password).catch(() => {throw new Error("authentication timeout")});
        } else {
            this.session.adHoc(username).catch(() => {throw new Error("authentication timeout")});
        }
    };

    // TODO: temporary
    componentDidMount = () => {
        console.debug("mounted -> auto connect")
        this.session.connect();
    };

    private connectionView = () => {
        if (this.state.sessionDescription) {
            return (
                <React.Fragment>
                    <aside>
                        <h4>my rooms</h4>
                        <RoomSelector rooms={this.state.myrooms} onSelect={roomToSendTo => this.setState({ roomToSendTo })} />
                        <ul> {
                        (this.state.participantsByRoom[this.state.roomToSendTo] || [])
                            .map(participant => <li key={participant.sessionId}>{participant.fullName}</li>)
                        }</ul>
                    </aside>

                    <section>
                        <div className="messageBox">
                            <div>
                                <MessageList
                                    messages={this.state.receivedMessagesByRoom[this.state.roomToSendTo] || []}
                                    me={this.state.sessionDescription.sessionId}/>
                                <span ref={e => this.lastMessage = !e ? undefined : e} ></span>
                            </div>
                        </div>
                        <SendForm onSend={this.sendToRoom} />
                    </section>

                    <nav>
                        <small>
                            <dl>
                                <dt>Username</dt>
                                <dd>{this.state.profile && this.state.profile.fullName}</dd>
                                <dt>SessionId</dt>
                                <dd><pre>{this.state.sessionDescription.sessionId}</pre></dd>
                            </dl>

                        </small>

                        <AuthenticatedView authenticated={this.state.authenticated} onsubmit={(u, p) => this.login(u, p)}>
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
                {this.connectionView()}
            </div>
        )
    }
}