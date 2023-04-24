import "./main.css";
import { answer } from "./answer";
function render() {
	document.getElementById(
		"root"
	).innerHTML = `<h1 class="text-3xl font-bold underline">the answer to the universe is ${answer}</h1>`;
}
render();
