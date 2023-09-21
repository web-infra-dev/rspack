import Button from "./Button";

onmessage = e => {
	Button.add();
	postMessage(Button.get());
};
