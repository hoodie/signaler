import * as React from "react";

import { Config } from ".";
import { Session } from '../../client-lib/';

import { SessionView } from './SessionView';

interface AppProps {
    config?: Config;
}

interface AppState {
    session?: Session;
}

export class SignalerContainer extends React.Component<AppProps, AppState> {

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
            sessionView = <SessionView session={this.state.session} onDisconnect={this.deleteSession} config={this.props.config}/>
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
                <code>{JSON.stringify(this.props.config)}</code>
            </React.Fragment>
        )
    }
}
