const path = require("path");
const { pathToFileURL } = require("url");

const sourceFilename = path.resolve(
	"./configCases/module/import-meta-parser-options/disabled-fields.js"
);
const sourceDirname = path.dirname(sourceFilename);
const sourceUrl = pathToFileURL(sourceFilename).toString();
const { env, url, webpack } = import.meta;

export default {
	contextType: typeof import.meta.webpackContext,
	dirname: import.meta.dirname,
	envType: typeof import.meta.env,
	filename: import.meta.filename,
	globType: typeof import.meta.glob,
	hotType: typeof import.meta.webpackHot,
	main: import.meta.main,
	sourceDirname,
	sourceFilename,
	sourceUrl,
	url,
	destructuredEnvType: typeof env,
	destructuredUrl: url,
	destructuredWebpack: webpack,
	webpack
};
