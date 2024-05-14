import { pathToFileURL } from "url";

import isEsmFile from "./isEsmFile";

/**
 * Dynamically import files. It will make sure it's not being compiled away by TS/Rollup.
 */
export const dynamicImport = new Function("path", "return import(path)");

const crossImport = async <T = any>(
	path: string,
	cwd = process.cwd()
): Promise<T> => {
	if (isEsmFile(path, cwd)) {
		const url = pathToFileURL(path).href;
		const { default: config } = await dynamicImport(url);
		return config;
	} else {
		let result = require(path);
		// compatible with export default config in common ts config
		if (result && typeof result === "object" && "default" in result) {
			result = result.default || {};
		}
		return result;
	}
};

export default crossImport;
