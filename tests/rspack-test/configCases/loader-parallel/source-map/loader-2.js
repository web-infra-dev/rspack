module.exports = function (content, sourceMap, additionalData) {
	this.callback(
		null,
		`module.exports = ${JSON.stringify({
			...sourceMap,
			sources: ["index.js"],
			mappings: "AAAA"
		})}`
	);
};
