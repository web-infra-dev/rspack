import { A, B } from "./b";

const obj = /*#__PURE__*/ (() => {
	return B;
})();

const obj2 = (() => {
	console.log(A);
})();