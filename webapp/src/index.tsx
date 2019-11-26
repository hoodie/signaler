import * as React from "react";
import * as ReactDOM from "react-dom";

import { Session } from "../../client-lib/";
import { SignalerContainer } from "./App";

export interface Config {
    username: string | null;
    password: string | null;
}

const config: Config = (() => {
    const { username, password } = Object.fromEntries(
        new URLSearchParams(window.location.search).entries()
    );
    return { username: username || null, password: password || null };
})();
console.table(config);

ReactDOM.render(
    <SignalerContainer config={config} />,
    document.querySelector("#app")
);

(window as any).initSession = () => {
    const session = new Session(`ws://${location.host}/ws/`);
};
