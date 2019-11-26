
import * as React from 'react';

export interface AuthenticatedViewProps {
    authenticated: boolean,
    onsubmit: Submitter;
}

export interface AuthenticatedViewState {
    username?: string,
    password?: string
}

export type Submitter = (username: string, password: string) => void;

export class AuthenticatedView extends React.Component<AuthenticatedViewProps, AuthenticatedViewState> {
    private username?: HTMLInputElement ;
    private password?: HTMLInputElement ;

    constructor(props: AuthenticatedViewProps) {
        super(props)
    }
    
    handleUsername = ({ target: { value: username} }: React.ChangeEvent<HTMLInputElement>) => 
        this.setState({ username });
    
    handlePassword = ({ target: { value: password} }: React.ChangeEvent<HTMLInputElement>) => 
        this.setState({ password });

    send = () => {
        this.props.onsubmit(this.state.username!, this.state.password!);
        this.username!.value = "";
        this.password!.value = "";
    }


    render() {
        return this.props.authenticated
            ? <div> {this.props.children} </div>
            : <div>
                <h6>authentication</h6>
                <fieldset>
                    <input type="text" name="username" id="username"
                        placeholder="username"
                        onChange={this.handleUsername}
                        ref={e => (this.username = e ? e : undefined)}
                        onKeyPress={e => e.key === "Enter" && this.send()}
                    />

                    <input type="password" name="password" id="password"
                        placeholder="password"
                        onChange={this.handlePassword}
                        ref={e => (this.password = e ? e : undefined)}
                        onKeyPress={e => e.key === "Enter" && this.send()}
                    />
                    <button onClick={this.send}>login</button>

                </fieldset>
            </div>;
    }
}