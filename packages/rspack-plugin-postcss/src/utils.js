/**
 * fork from [postcss-loader](https://github.com/webpack-contrib/postcss-loader/blob/d274f9007c8d2db63924e14c06188134d99d6d0d/src/utils.js)
 */

const path = require("path");

const IS_NATIVE_WIN32_PATH = /^[a-z]:[/\\]|^\\\\/i;
const ABSOLUTE_SCHEME = /^[a-z0-9+\-.]+:/i;

function getURLType(source) {
	if (source[0] === "/") {
		if (source[1] === "/") {
			return "scheme-relative";
		}

		return "path-absolute";
	}

	if (IS_NATIVE_WIN32_PATH.test(source)) {
		return "path-absolute";
	}

	return ABSOLUTE_SCHEME.test(source) ? "absolute" : "path-relative";
}

function normalizeSourceMap(map, resourceContext) {
	let newMap = map;

	// Some loader emit source map as string
	// Strip any JSON XSSI avoidance prefix from the string (as documented in the source maps specification), and then parse the string as JSON.
	if (typeof newMap === "string") {
		newMap = JSON.parse(newMap);
	}

	delete newMap.file;

	const { sourceRoot } = newMap;

	delete newMap.sourceRoot;

	if (newMap.sources) {
		newMap.sources = newMap.sources.map(source => {
			const sourceType = getURLType(source);

			// Do no touch `scheme-relative` and `absolute` URLs
			if (sourceType === "path-relative" || sourceType === "path-absolute") {
				const absoluteSource =
					sourceType === "path-relative" && sourceRoot
						? path.resolve(sourceRoot, path.normalize(source))
						: path.normalize(source);

				return path.relative(resourceContext, absoluteSource);
			}

			return source;
		});
	}

	return newMap;
}

function normalizeSourceMapAfterPostcss(map, resourceContext) {
	const newMap = map;

	// result.map.file is an optional property that provides the output filename.
	// Since we don't know the final filename in the webpack build chain yet, it makes no sense to have it.
	delete newMap.file;

	newMap.sourceRoot = "";

	newMap.sources = newMap.sources.map(source => {
		if (source.indexOf("<") === 0) {
			return source;
		}

		const sourceType = getURLType(source);

		// Do no touch `scheme-relative`, `path-absolute` and `absolute` types
		if (sourceType === "path-relative") {
			return path.resolve(resourceContext, source);
		}

		return source;
	});

	return newMap;
}

module.exports = {
	normalizeSourceMapAfterPostcss,
	normalizeSourceMap
};
