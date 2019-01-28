import * as React from 'react';
import * as ReactDOM from 'react-dom';

import { Session, SessionDescription } from '../../client-lib/';

interface SessionViewProps {
    session: Session;
    onDisconnect: () => void;
}

interface SessionViewState {
    selectedChannel: string;
    sessionDescription?: SessionDescription;
}

class SessionView extends React.Component<SessionViewProps, SessionViewState> {
    private session: Session;

    constructor(props: SessionViewProps) {
        super(props);
        this.state = {
            selectedChannel: "default"
        };
        this.session = props.session;

        this.session.onWelcome.add(sessionDescription => this.setState({ sessionDescription }));
        this.session.onConnectionClose.add(event => {
            console.error(event);
            this.props.onDisconnect();
        });
    }

    private selectChannel: React.ChangeEventHandler<HTMLInputElement> = ({ target: { value } }) => {
        this.setState({ selectedChannel: value })
    };

    private connectionView = () => {
        if (this.state.sessionDescription) {
            return (
                <div>
                    <input
                        type="text" name="channelName" id="channelName"
                        onChange={this.selectChannel} placeholder={this.state.selectedChannel} />

                    <button onClick={() => this.session.join(this.state.selectedChannel)}> join </button>
                    <hr />
                    <button onClick={this.disconnect}> disconnect </button>
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

interface AppState {
    session?: Session;
}

class App extends React.Component<{}, AppState> {

    constructor(props: any) {
        super(props);
        this.state = {
            session: undefined
        };
    }

    private createSession = () => {
        const session = new Session('ws://localhost:8080/ws/')
        this.setState({ session });
        console.debug("creating session", session);
    };

    private deleteSession = () => {
        this.setState({ session: undefined })
    };

    render() {
        let sessionView;
        if (!!this.state.session) {
            sessionView = <SessionView session={this.state.session} onDisconnect={this.deleteSession}/>
        } else {
            sessionView = <div>no session yet</div>;
        }

        return (
            <React.Fragment>
                <h2>Signaler</h2>
                <button onClick={this.createSession}> create new Session</button>
                {sessionView}
            </React.Fragment>
        )
    }
}

ReactDOM.render(<App/>, document.querySelector('#app'));

(window as any).initSession = () => {
    const session = new Session('ws://localhost:8080/ws/');
};