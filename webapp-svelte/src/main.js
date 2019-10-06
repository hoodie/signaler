import App from './App.svelte';

const app = new App({
	target: document.body,
	props: {
		name: 'world',
		url: `ws://${location.host}/ws/`,
	}
});

window.app = app;

export default app;