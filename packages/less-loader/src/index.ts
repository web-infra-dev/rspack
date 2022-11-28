import type { LoaderContext, Loader } from "@rspack/core";
import path from "path";
import { create } from "enhanced-resolve";
import assert from "node:assert";
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

const lessLoader: Loader = async function (content) {
	assert(typeof content === "string");

	const callback = this.async();
	const enhancedResolver = create.sync({
		extensions: [".less", ".css", ".sass", ".scss", ".js"],
		preferRelative: true
	});
	const options: Options = this.getOptions() ?? {};
	const lessOptions = options.lessOptions ?? {};
	const useSourceMap =
		typeof options.sourceMap === "boolean" ? options.sourceMap : this.sourceMap;

	if (useSourceMap) {
		lessOptions.sourceMap = {
			outputSourceFiles: true
		};
	}

	try {
		let data = content;
		if (typeof options.additionalData !== "undefined") {
			data =
				typeof options.additionalData === "function"
					? `${await options.additionalData(data, this)}`
					: `${options.additionalData}\n${data}`;
		}
		const resolver = (id, dir): string => {
			let request = id;
			// convert '~a/b/c' to 'a/b/c'
			// from https://github.com/webpack-contrib/less-loader/blob/master/src/utils.js#L73
			if (MODULE_REQUEST_REGEX.test(id)) {
				request = request.replace(MODULE_REQUEST_REGEX, "");
			}
			try {
				return enhancedResolver(dir, request) as string;
			} catch (err) {
				throw err;
			}
		};
		const final_options = {
			filename: this.resourcePath,
			...lessOptions,
			paths: [
				...(lessOptions?.paths || ["node_modules"]),
				path.dirname(this.resourcePath)
			],
			plugins: [
				new LessAliasesPlugin({
					config: {
						resolve: resolver
					},
					stdinDir: path.dirname(this.resourcePath)
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

		callback(null, result.css, map);
	} catch (error) {
		console.log(this.resourcePath);
		console.log(error);
		callback(error, "");
	}
};

module.exports = lessLoader;
