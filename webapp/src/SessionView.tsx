import * as React from 'react';

import { Session, SessionDescription } from '../../client-lib/';

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

interface SendFormProps { onSend: Fn<string> }

class SendForm extends React.Component<SendFormProps, { content: string }> {
    constructor(props: SendFormProps) {
        super(props);
    }

    handleContent = ({ target: { value: content } }: React.ChangeEvent<HTMLInputElement>) => {
      this.setState({ content })
    };

    render() {
        return (
            <fieldset>
                <input type="text" onChange={this.handleContent} id="" />
                <button onClick={() => this.props.onSend(this.state.content)}>send</button>
            </fieldset>
        );
    }
}

export interface SessionViewProps {
    session: Session;
    onDisconnect: () => void;
}

interface SessionViewState {
    rooms: string[];
    myrooms: string[];
    roomToSendTo: string;
    roomToJoin: string;
    receivedMessagesByRoom: { [index: string]: string[] },

    sessionDescription?: SessionDescription;
}

export class SessionView extends React.Component<SessionViewProps, SessionViewState> {
    private session: Session;

    constructor(props: SessionViewProps) {
        super(props);
        this.state = {
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
        });

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
            receivedMessagesByRoom[room] = [...oldRoomMessages, message.content]
            console.debug({receivedMessagesByRoom})
            this.setState({ receivedMessagesByRoom })
        });

        (window as any).session = this.session;
    }

    private roomToJoin: React.ChangeEventHandler<HTMLInputElement> = ({ target: { value } }) => {
        this.setState({ roomToJoin: value })
    };

    private sendToRoom = (content: string) => {
        this.session.sendMessage(content, this.state.roomToSendTo)
    };

    private connectionView = () => {
        if (this.state.sessionDescription) {
            return (
                <div>
                    <fieldset>

                        <label>
                            join
                            <RoomSelector rooms={this.state.rooms} onSelect={room => { this.session.join(room) }} />
                        </label>

                        <label>
                            <input
                                type="text" name="channelName" id="channelName"
                                onChange={this.roomToJoin} placeholder="createNew" />
                            <button onClick={() => this.session.join(this.state.roomToJoin)}> join </button>
                        </label>
                        <hr />
                        <button onClick={this.disconnect}> disconnect </button>
                    </fieldset>
                    <div>
                        my rooms
                        <RoomSelector rooms={this.state.myrooms} onSelect={ roomToSendTo => this.setState({ roomToSendTo }) } />
                        <ul>{
                            (this.state.receivedMessagesByRoom[this.state.roomToSendTo] || []).map(msg => <li key={msg}>{msg}</li>)

                        }</ul>
                        <SendForm onSend={this.sendToRoom} />
                    </div>
                </div>
            )
        } else {
            return <React.Fragment />
        }
    };

    private disconnect = () => {
        this.session.disconnect();
        this.props.onDisconnect();
    }

    render() {
        return (
            <div>
                <h3>Session</h3>
                <small> {this.state.sessionDescription &&
                    this.state.sessionDescription.uuid}
                </small>
                <button onClick={() => this.session.connect()}> connect </button>
                {this.connectionView()}
            </div>
        )
    }
}