const path = require("path");

const loaderRuns = {
	left: 0,
	marked: 0,
	right: 0,
	"source-map-consumer": 0,
	"source-map-producer": 0
};

module.exports = function (source, sourceMap) {
	const { name } = this.getOptions();
	if (name === "module-id") {
		return source.replace("__MODULE_ID__", path.basename(this.resourcePath));
	}
	if (name === "source-map-producer") {
		loaderRuns[name]++;
		this.callback(null, source, {
			version: 3,
			sources: [`value-${loaderRuns[name]}.js`],
			names: [],
			mappings: ""
		});
		return;
	}
	if (name === "source-map-consumer") {
		loaderRuns[name]++;
		const result = source
			.replace("__SOURCE_MAP_CONSUMER__", loaderRuns[name])
			.replace("__SOURCE_MAP_INPUT__", JSON.stringify(sourceMap?.sources[0]));
		this.callback(null, result, sourceMap);
		return;
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
