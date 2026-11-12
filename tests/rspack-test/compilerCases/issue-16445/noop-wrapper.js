/*#__NO_SIDE_EFFECTS__*/
function annotatedLeaf(callback) {
	/*#__PURE__*/ callback();
}

export function check(callback) {
	annotatedLeaf(() => callback());
}
