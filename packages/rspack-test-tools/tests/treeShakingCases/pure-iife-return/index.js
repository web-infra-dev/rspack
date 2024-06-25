import { A, B } from "./b";

let obj = /*#__PURE__*/ (() => {
	return B;
})();

let obj2 = (() => {
	console.log(A);
})();