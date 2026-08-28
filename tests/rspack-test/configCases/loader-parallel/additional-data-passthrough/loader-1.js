module.exports = function (content, sourceMap, additionalData) {
	this.callback(null, content, null, {
		a: "a",
		buffer: Buffer.from("native-owned"),
		map: new Map([["key", "value"]])
	});
};
