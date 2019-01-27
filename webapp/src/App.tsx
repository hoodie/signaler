import * as React from 'react';
import * as ReactDOM from 'react-dom';

import { handleMessage, hiThere } from '../../client-lib/';

const App = (props: {name?: string}) => {
    return <h3>App {props.name}</h3>
};

ReactDOM.render(<App name="signaler"/>, document.querySelector('#app'));