import { myanswer, secret } from "./lib";

export function render() {
	function test() {
		const container = document.getElementById("root");
		container.innerHTML = `adddd333:${secret}:${myanswer}`;
	}
}

export default function result() {}
