import { answer } from "./answer";
let a = 10;
function render() {
	document.getElementById(
		"root"
	).innerHTML = `the answer to the universe is ${answer}`;
}
render();
