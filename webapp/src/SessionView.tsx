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

  handleContent = ({target: {value: content}}: React.ChangeEvent<HTMLTextAreaElement>) => {
      this.setState({ content })
  };

    render() {
        return (
            <fieldset>
                <textarea name="" onChange={this.handleContent} id="" cols={30} rows={10} />
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

    sessionDescription?: SessionDescription;
}

export class SessionView extends React.Component<SessionViewProps, SessionViewState> {
    private session: Session;

    constructor(props: SessionViewProps) {
        super(props);
        this.state = {
            rooms: [],
            myrooms: [],
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