const path = require("path");

const loaderRuns = {
	left: 0,
	marked: 0,
	right: 0,
	"bom-consumer": 0,
	"bom-producer": 0
};

module.exports = function (source, sourceMap) {
	const { name } = this.getOptions();
	if (name === "module-id") {
		return source.replace("__MODULE_ID__", path.basename(this.resourcePath));
	}
	if (name === "bom-producer") {
		loaderRuns[name]++;
		return `\uFEFF${source.replace("__BOM_PRODUCER__", loaderRuns[name])}`;
	}
	if (name === "bom-consumer") {
		loaderRuns[name]++;
		this.addDependency(path.join(path.dirname(this.resourcePath), "value.js"));
		return source
			.replace("__HAS_BOM__", source.charCodeAt(0) === 0xfeff)
			.replace("__BOM_CONSUMER__", loaderRuns[name]);
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
