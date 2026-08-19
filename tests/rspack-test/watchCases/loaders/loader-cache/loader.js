const path = require("path");

const loaderRuns = {
	left: 0,
	marked: 0,
	right: 0
};

module.exports = function (source, sourceMap) {
	const { name } = this.getOptions();
	if (name === "metadata") {
		const match = source.match(
			/^\/\/ loader-cache:options=([^;]+);version=([^\n]+)\n/
		);
		if (!match) throw new Error("Missing loader cache test metadata");
		// Change etag-only fields while keeping the cached loader's input stable.
		const target = this.loaders[this.loaderIndex - 1].loaderItem;
		target.optionsCacheKey = match[1];
		target.loaderVersion = match[2];
		return source.slice(match[0].length);
	}
	if (name === "module-id") {
		return source.replace("__MODULE_ID__", path.basename(this.resourcePath));
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
	if (name === "marked") {
		this.callback(null, source, sourceMap);
		return;
	}

	return source.replace("__SOURCE_MAP__", sourceMap?.version === 3);
};
