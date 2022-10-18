import path from "path";
import { normalizeSourceMap } from "./utils";

const generateOptions = (options: Less.Options): Less.Options => {
	const defaultConfig = {
		enableSourcemap: false
	};
	return {
		...defaultConfig,
		...options
	};
};

export interface Options {
	implementation?: string;
	lessOptions?: Less.Options;
	sourceMap?: boolean,
}

export default async function lessLoader(loaderContext) {
	let meta = "";
	const options: Options = loaderContext.getOptions() ?? {};
	const lessOptions = options.lessOptions ?? {};
	const useSourceMap =
		typeof options.sourceMap === "boolean"
			? options.sourceMap
			: loaderContext.useSourceMap;

	if (useSourceMap) {
		lessOptions.sourceMap = {
			outputSourceFiles: true
		};
	}

	try {
		let code = loaderContext.source.getCode();
		const final_options = generateOptions({
			filename: loaderContext.resourcePath,
			...lessOptions,
			paths: [
				...(lessOptions?.paths || ["node_modules"]),
				path.dirname(loaderContext.resourcePath)
			],
			plugins: []
		});

		// eslint-disable-next-line import/no-dynamic-require, global-require
		let lessImplementation;

		if (typeof options.implementation === "string") {
			lessImplementation = require(options.implementation);
		} else {
			lessImplementation = (await import("less")).default;
		}
		const result = await lessImplementation.render(code, final_options);
		let map =
			typeof result.map === "string" ? JSON.parse(result.map) : result.map;

		if (map && useSourceMap) {
			map = normalizeSourceMap(map);
		}

		return {
			content: result.css,
			meta: meta ? Buffer.from(JSON.stringify(meta)) : "",
			sourceMap: map
		};
	} catch (error) {
		console.log(loaderContext.resourcePath);
		console.log(error);
		throw error;
	}
}
