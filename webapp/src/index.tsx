import * as React from "react";
import * as ReactDOM from "react-dom";

import { Session } from "../../client-lib/";
import { SignalerContainer } from "./App";

export interface Config {
    username: string | null;
    password: string | null;
    cli: string | null;
}

const defaultConfig: Config = {
    username: null,
    password: null,
    cli: null
};

const config: Config = {
    ...defaultConfig,
    ...Object.fromEntries(new URLSearchParams(window.location.search).entries())
};

console.table(config);

function initApp() {
    ReactDOM.render(
        <SignalerContainer config={config} />,
        document.querySelector("#app")
    );
}

function initCli() {
    const session = new Session(`ws://${location.host}/ws/`);
    (window as any)["session"] = session;
    console.info('use `session`', session);
}

(window as any)["initApp"] = initApp;
(window as any)["initCli"] = initCli;

if (config.cli === "true") {
    initCli();
} else {
    initApp();
}
