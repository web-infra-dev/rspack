import fs from "node:fs";
import { sync as resolveSync, transformSync } from "@rspack/binding";
import type { Compiler } from "../Compiler";

type BrowserRequire = typeof Compiler.prototype.__internal_browser_require;

interface CommonJsRuntime {
	module: any;
	exports: any;
	require: BrowserRequire;
}

interface BrowserRequirePluginOptions {
	execute: (code: string, runtime: CommonJsRuntime) => void;
}

const unsafeExecute: BrowserRequirePluginOptions["execute"] = (
	code,
	runtime
) => {
	const wrapper = new Function("module", "exports", "require", code);
	wrapper(runtime.module, runtime.exports, runtime.require);
};

/**
 * Loading modules in `@rspack/browser` is different from `@rspack/core`.
 * 1. It resolves the JavaScript in the memfs with Node.js resolution algorithm rather than in the host filesystem.
 * 2. It customizes how to evaluate CJS/ESM because there's no `require` any more.
 */
export class BrowserRequirePlugin {
	static unsafeExecute = unsafeExecute;

	constructor(private options: BrowserRequirePluginOptions) {}

	apply(compiler: Compiler) {
		const execute = this.options.execute;
		compiler.__internal_browser_require = function browserRequire(id: string) {
			const { path: loaderPath } = resolveSync("", id);
			if (!loaderPath) {
				throw new Error(`Cannot find loader of ${id}`);
			}

			const data = fs.readFileSync(loaderPath);
			const code = data?.toString() || "";

			const module: any = { exports: {} };
			const exports = module.exports;

			const cjs = transformSync(
				code,
				JSON.stringify({
					module: { type: "commonjs" }
				})
			);

			execute(cjs.code, { exports, module, require: browserRequire });
			return exports.default || module.exports;
		};
	}
}
