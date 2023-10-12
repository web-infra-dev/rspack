import { myanswer, secret } from "./lib1";

setTimeout(() => {
	console.log(myanswer);
}, 1000);

export function render() {
	function test() {
		const container = document.getElementById("root");
		container.innerHTML = `adddd333:${secret}:${myanswer}${fun}`;
	}
}

function fun() {

}
