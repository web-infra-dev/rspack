import { answer } from "./answer";
function render() {
	document.getElementById(
		"root"
	).innerHTML = `the answer to the universe is ${answer}`;
}
console.log(process.env.test)
render();
