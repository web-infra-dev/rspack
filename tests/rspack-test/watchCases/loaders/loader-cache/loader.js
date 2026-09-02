const path = require("path");
const fs = require("fs");

const loaderRuns = {
	left: 0,
	marked: 0,
	right: 0,
	"bom-consumer": 0,
	"bom-producer": 0,
	"file-dependency": 0,
	"context-dependency": 0,
	"context-downstream": 0,
	"build-dependency": 0,
	"missing-dependency": 0,
	"chain-left": 0
};

module.exports = function (source, sourceMap) {
	const { name: configuredName } = this.getOptions();
	const name = configuredName === "dependency"
		? path.basename(this.resourcePath, path.extname(this.resourcePath))
		: configuredName;
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
	if (name === "file-dependency") {
		loaderRuns[name]++;
		const dependency = path.join(path.dirname(this.resourcePath), "file-dependency.txt");
		this.addDependency(dependency);
		return source
			.replace("__DEPENDENCY_VALUE__", JSON.stringify(fs.readFileSync(dependency, "utf-8").trim()))
			.replace("__DEPENDENCY_RUNS__", loaderRuns[name]);
	}
	if (name === "context-dependency") {
		loaderRuns[name]++;
		const dependency = path.join(path.dirname(this.resourcePath), "context-dependency");
		this.addContextDependency(dependency);
		return source
			.replace("__DEPENDENCY_VALUE__", JSON.stringify(fs.readdirSync(dependency).sort()))
			.replace("__DEPENDENCY_RUNS__", loaderRuns[name]);
	}
	if (name === "context-downstream") {
		loaderRuns[name]++;
		return source.replace(
			/(\bruns:\s*\d+)/,
			`$1,\n\tdownstreamRuns: ${loaderRuns[name]}`
		);
	}
	if (name === "build-dependency") {
		loaderRuns[name]++;
		const dependency = path.join(path.dirname(this.resourcePath), "build-dependency.txt");
		this.addBuildDependency(dependency);
		this.addDependency(dependency);
		return source
			.replace("__DEPENDENCY_VALUE__", JSON.stringify(fs.readFileSync(dependency, "utf-8").trim()))
			.replace("__DEPENDENCY_RUNS__", loaderRuns[name]);
	}
	if (name === "missing-dependency") {
		loaderRuns[name]++;
		const dirname = path.dirname(this.resourcePath);
		const trigger = path.join(dirname, "missing-trigger.txt");
		this.addDependency(trigger);
		this.addMissingDependency(path.join(dirname, "does-not-exist.txt"));
		return source
			.replace("__DEPENDENCY_VALUE__", JSON.stringify(fs.readFileSync(trigger, "utf-8").trim()))
			.replace("__DEPENDENCY_RUNS__", loaderRuns[name]);
	}
	if (name === "chain-left") {
		loaderRuns[name]++;
		this.addDependency(path.join(path.dirname(this.resourcePath), "chain-left.txt"));
		return source.replace("__CHAIN_LEFT_RUNS__", loaderRuns[name]);
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
