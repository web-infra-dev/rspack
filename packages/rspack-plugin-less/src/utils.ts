/**
 * fork from [less-loader](https://github.com/webpack-contrib/less-loader/blob/66f28a0aaea750f8aaa1aae59d9b0f721ba8c183/src/utils.js)
 */

import * as path from "path";

export function normalizeSourceMap(map) {
	const newMap = map;

	// map.file is an optional property that provides the output filename.
	// Since we don't know the final filename in the webpack build chain yet, it makes no sense to have it.
	delete newMap.file;

	newMap.sourceRoot = "";

	// `less` returns POSIX paths, that's why we need to transform them back to native paths.
	newMap.sources = newMap.sources.map((source) => path.normalize(source));

	return newMap;
}
