const path = require("path");
const { pathToFileURL } = require("url");

const sourceUrl = pathToFileURL(
	path.resolve("./configCases/module/import-meta-parser-options/empty-options.js")
).toString();

if (!import.meta.UNKNOWN_PROPERTY) {
	import.meta.UNKNOWN_PROPERTY = "runtime";
}

const { UNKNOWN_PROPERTY, url, webpack } = import.meta;

export default {
	sourceUrl,
	unknown: UNKNOWN_PROPERTY,
	unknownOptional: import.meta.UNKNOWN_PROPERTY?.length,
	missingOptional: import.meta.MISSING_PROPERTY?.length,
	url,
	webpack
};
