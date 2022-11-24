module.exports = function (content) {
	return (
		"module.exports = " +
		JSON.stringify({
			resource: this.resource,
			prev: content
		})
	);
};
