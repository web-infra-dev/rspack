module.exports = function (content, sourceMap, additionalData) {
	this.callback(
		null,
		`module.exports = ${JSON.stringify({
			a: additionalData.a,
			buffer: Buffer.isBuffer(additionalData.buffer) && additionalData.buffer.toString(),
			map: additionalData.map instanceof Map && additionalData.map.get("key"),
			b: "b"
		})}`,
		null
	);
};
