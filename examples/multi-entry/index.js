import { something } from "./static";
const a = 10;
async function render() {
	import("./answer").then(x => {
		console.log("x:", x);
	});
	console.log("a:", a);
	console.log("static:", something);
	document.getElementById("root").innerHTML = a;
}

render();
