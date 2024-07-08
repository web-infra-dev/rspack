module.exports = function (content) {
	return (
		"module.exports = " +
		JSON.stringify({
			resourceQuery: this.resourceQuery,
			prev: content
		})
	);
};
