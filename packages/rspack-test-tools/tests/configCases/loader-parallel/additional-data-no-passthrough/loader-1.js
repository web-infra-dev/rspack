module.exports = function (content, sourceMap, additionalData) {
	this.callback(null, content, null, {
		a: "a"
	});
};
