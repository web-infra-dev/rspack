import { something } from "./static";
const a = 10;
async function render() {
	const { answer } = await import("./answer");
	console.log("a:", a);
	console.log("answer:", answer);
	console.log("static:", something);
	document.getElementById("root").innerHTML = answer;
}

render();
