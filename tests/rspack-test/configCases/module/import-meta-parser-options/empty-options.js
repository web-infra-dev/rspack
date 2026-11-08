const path = require("path");
const { pathToFileURL } = require("url");

const sourceUrl = pathToFileURL(
	path.resolve("./configCases/module/import-meta-parser-options/empty-options.js")
).toString();

if (!import.meta.UNKNOWN_PROPERTY) {
	import.meta.UNKNOWN_PROPERTY = "runtime";
}

let computedAccesses = 0;
const getUnknownProperty = () => {
	computedAccesses++;
	return "UNKNOWN_PROPERTY";
};

const { UNKNOWN_PROPERTY, url, webpack } = import.meta;

export default {
	sourceUrl,
	unknown: UNKNOWN_PROPERTY,
	computedUnknown: import.meta[getUnknownProperty()],
	templateUnknown: import.meta[`UNKNOWN_PROPERTY`],
	computedAccesses,
	unknownOptional: import.meta.UNKNOWN_PROPERTY?.length,
	missingOptional: import.meta.MISSING_PROPERTY?.length,
	url,
	webpack
};
