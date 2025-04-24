module.exports = function (content, sourceMap, additionalData) {
	this.callback(
		null,
		`module.exports = { value: ${content} }`,
		null,
		null
	);
};
