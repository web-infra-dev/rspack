import App from "./App.svelte";

const loadApp = (id, name) => {
	return new App({
		target: document.querySelector(`#${id}`),
		props: { name }
	});
};

export default loadApp;
