module.exports = function (content, sourceMap, additionalData) {
	this.callback(
		null,
		`module.exports = ${JSON.stringify({
			...additionalData,
			workerLocal: additionalData.workerLocal.description,
			b: "b"
		})}`,
		null
	);
};
