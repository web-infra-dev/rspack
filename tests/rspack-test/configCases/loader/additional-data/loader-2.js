module.exports = function (content, sourceMap, additionalData) {
	this.callback(
		null,
		`module.exports = ${JSON.stringify({
			...additionalData,
			b: "b"
		})}`,
		null
	);
};
