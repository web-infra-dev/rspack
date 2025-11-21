module.exports = function (content) {
	return (
		"module.exports = " +
		JSON.stringify({
			resourcePath: this.resourcePath,
			prev: content
		})
	);
};
