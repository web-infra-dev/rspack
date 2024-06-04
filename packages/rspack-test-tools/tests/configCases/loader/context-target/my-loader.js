module.exports = function (content) {
	return (
		"module.exports = " +
		JSON.stringify({
			target: this.target,
			prev: content
		})
	);
};
