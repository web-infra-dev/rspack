import type { Compiler } from "@rspack/core";

/**
 * @typedef {Object} Options
 * @property {(string | RegExp | (string | RegExp)[] | null)=} include included resourcePath for loader
 * @property {(string | RegExp | (string | RegExp)[] | null)=} exclude excluded resourcePath for loader
 */
export default class ReactRefreshRspackPlugin {
	/**
	 * @param {Options} options
	 */
	constructor(options?: {
		include?: string | RegExp | (string | RegExp)[] | null;
		exclude?: string | RegExp | (string | RegExp)[] | null;
	});
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler: Compiler): void;

	/**
	 * @deprecated
	 */
	static deprecated_runtimePaths: string;
}
