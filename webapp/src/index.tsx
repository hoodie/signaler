import * as React from "react";
import * as ReactDOM from 'react-dom';

import { Session } from '../../client-lib/';
import { SignalerContainer } from './App';

ReactDOM.render(<SignalerContainer/>, document.querySelector('#app'));

(window as any).initSession = () => {
      const session = new Session(`ws://${location.host}/ws/`)
};
