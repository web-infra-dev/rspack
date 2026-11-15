let count = 0;

function track(value) {
	count++;
	return value;
}

module.exports = {
	used: "used",
	unusedPure: /*#__PURE__*/ track("unused-pure"),
	unusedLiteral: track,
	getCount() {
		return count;
	}
};
