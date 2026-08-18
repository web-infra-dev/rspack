const loaderRuns = {
	left: 0,
	marked: 0,
	right: 0
};

module.exports = function (source, sourceMap) {
	const { name } = this.getOptions();
	if (name === "marked") {
		this.addDependency(`${this.rootContext}/trigger.js`);
	}
	loaderRuns[name]++;
	source = source.replace(`__${name.toUpperCase()}__`, loaderRuns[name]);

	if (name === "right") {
		this.callback(
			null,
			source,
			{
				version: 3,
				sources: ["value.js"],
				names: [],
				mappings: ""
			}
		);
		return;
	}

	return source.replace("__SOURCE_MAP__", sourceMap?.version === 3);
};
