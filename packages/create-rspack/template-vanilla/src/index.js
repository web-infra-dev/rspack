import { message } from "./message";
function render() {
	const container = document.getElementById("root");
	console.log({ container });
	container.innerHTML = `Hello ${message}`;
}
render();
