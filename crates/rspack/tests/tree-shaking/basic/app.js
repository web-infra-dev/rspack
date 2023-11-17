import { myanswer, secret } from "./lib";

setTimeout(() => {
	console.log(myanswer);
}, 1000);

export function render() {
	function test() {
		const container = document.getElementById("root");
		container.innerHTML = `adddd333:${secret}:${myanswer}`;
	}
}

if (module.hot?.accept) {
	module.hot.accept((module) => {
		console.log("xxx:", module);
	});
}
