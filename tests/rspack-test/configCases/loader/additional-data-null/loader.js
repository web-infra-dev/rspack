module.exports = async function (content, sourceMap, additionalData) {
	this.callback(
		null,
		`module.exports = { value: ${content} }`,
		null,
		null
	);
};
