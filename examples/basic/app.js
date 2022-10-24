import { myanswer, secret } from "./lib";

// console.log("answer:", myanswer, secret);
// setTimeout(() => {
//   answer++;
// },1000)

export function render() {
	const container = document.getElementById("root");
	container.innerHTML = `secret:${secret}\nanswer:${myanswer}`;
}

// if (module.hot?.accept) {
// 	module.hot.accept((module) => {
// 		console.log("xxx:", module);
// 	});
// }
function result() {}

export default result;
// test();
render();
