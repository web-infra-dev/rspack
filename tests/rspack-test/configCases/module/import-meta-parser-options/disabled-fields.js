const path = require("path");
const { pathToFileURL } = require("url");

const sourceFilename = path.resolve(
	"./configCases/module/import-meta-parser-options/disabled-fields.js"
);
const sourceDirname = path.dirname(sourceFilename);
const sourceUrl = pathToFileURL(sourceFilename).toString();
const { url, webpack } = import.meta;

export default {
	contextType: typeof import.meta.webpackContext,
	dirname: import.meta.dirname,
	filename: import.meta.filename,
	globType: typeof import.meta.glob,
	hotType: typeof import.meta.webpackHot,
	main: import.meta.main,
	sourceDirname,
	sourceFilename,
	sourceUrl,
	url,
	urlOptional: import.meta.url?.length,
	webpackOptional: import.meta.webpack?.x,
	destructuredUrl: url,
	destructuredWebpack: webpack,
	webpack
};
