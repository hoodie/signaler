import * as React from "react";

import { Session } from '../../client-lib/';

import { SessionView } from './SessionView';

interface AppState {
    session?: Session;
}

export class App extends React.Component<{}, AppState> {

    constructor(props: any) {
        super(props);
        this.state = {
            session: undefined
        };
    }

    componentDidMount() {
        this.createSession();
    }

    private createSession = () => {
        const session = new Session(`ws://${location.host}/ws/`)
        this.setState({ session });
        console.debug("creating session", session);
    };

    private deleteSession = () => {
        this.setState({ session: undefined })
    };

    render() {
        let sessionView;
        if (!!this.state.session) {
            sessionView = <SessionView session={this.state.session} onDisconnect={this.deleteSession} />
        } else {
            sessionView = <div>
                no session yet
                <button onClick={this.createSession}> create new Session</button>
            </div>;
        }

        return (
            <React.Fragment>
                <h2>Signaler</h2>
                {sessionView}
            </React.Fragment>
        )
    }
}
