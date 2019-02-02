import * as React from "react";
import * as ReactDOM from 'react-dom';

import { Session } from '../../client-lib/';
import { App } from './App';

ReactDOM.render(<App/>, document.querySelector('#app'));

(window as any).initSession = () => {
    const session = new Session('ws://localhost:8080/ws/');
};