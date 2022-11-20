import type { LoaderContext } from "@rspack/core";
import path from "path";
import LessAliasesPlugin from "./alias-plugin";
import { normalizeSourceMap } from "./utils";
const MODULE_REQUEST_REGEX = /^[^?]*~/;
export interface Options {
	implementation?: string;
	lessOptions?: Less.Options;
	sourceMap?: boolean;
	additionalData?:
		| string
		| ((content: string, loaderContext: LoaderContext) => string)
		| ((content: string, loaderContext: LoaderContext) => Promise<string>);
}

export default async function lessLoader(loaderContext: LoaderContext) {
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
		let data = loaderContext.source.getCode();
		if (typeof options.additionalData !== "undefined") {
			data =
				typeof options.additionalData === "function"
					? `${await options.additionalData(data, loaderContext)}`
					: `${options.additionalData}\n${data}`;
		}
		const resolver = (id, dir) => {
			let request = id;
			// convert '~a/b/c' to 'a/b/c'
			// from https://github.com/webpack-contrib/less-loader/blob/master/src/utils.js#L73
			if (MODULE_REQUEST_REGEX.test(id)) {
				request = request.replace(MODULE_REQUEST_REGEX, "");
			}
			return require.resolve(request, { paths: [dir] });
		};
		const final_options = {
			filename: loaderContext.resourcePath,
			...lessOptions,
			paths: [
				...(lessOptions?.paths || ["node_modules"]),
				path.dirname(loaderContext.resourcePath)
			],
			plugins: [
				new LessAliasesPlugin({
					config: {
						resolve: resolver
					},
					stdinDir: path.dirname(loaderContext.resourcePath)
				})
			],
			relativeUrls: true
		};

		let lessImplementation;

		if (typeof options.implementation === "string") {
			lessImplementation = require(options.implementation);
		} else {
			lessImplementation = (await import("less")).default;
		}
		const result = await lessImplementation.render(data, final_options);
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
