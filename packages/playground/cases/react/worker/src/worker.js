import Button from "./Button";

window.onmessage = () => {
	Button.add();
	postMessage(Button.get());
};
