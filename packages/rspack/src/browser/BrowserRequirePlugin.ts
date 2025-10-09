import fs from "node:fs";
import { sync as resolveSync, transformSync } from "@rspack/binding";
import type { Compiler } from "../Compiler";

type BrowserRequire = typeof Compiler.prototype.__internal_browser_require;

/**
 * Represents the runtime context for CommonJS modules in a browser environment.
 */
interface CommonJsRuntime {
	module: any;
	exports: any;
	require: BrowserRequire;
}

interface BrowserRequirePluginOptions {
	/**
	 * This function defines how to execute CommonJS code.
	 */
	execute?: (code: string, runtime: CommonJsRuntime) => void;
	/**
	 * This option provides a direct mapping from the module specifier to the module content, similar to the mechanism of a virtual module.
	 * If this option is not provided or the mapping result is undefined, it will fallback to resolving from memfs and run `execute`.
	 */
	modules?: Record<string, unknown> | ((id: string) => unknown);
}

const unsafeExecute: BrowserRequirePluginOptions["execute"] = (
	code,
	runtime
) => {
	const wrapper = new Function("module", "exports", "require", code);
	wrapper(runtime.module, runtime.exports, runtime.require);
};

/**
 * This plugin inject browser-compatible `require` function to the `Compiler`.
 * 1. This plugin makes it possible to use custom loaders in browser by providing a virtual module mechanism.
 * 2. This plugin resolves the JavaScript in the memfs with Node.js resolution algorithm rather than in the host filesystem.
 * 3. This plugin transform ESM to CommonJS which will be executed with a user-defined `execute` function.
 */
export class BrowserRequirePlugin {
	/**
	 * This is an unsafe way to execute code in the browser using `new Function`.
	 * It is your responsibility to ensure that your application is not vulnerable to attacks due to this function.
	 */
	static unsafeExecute = unsafeExecute;

	constructor(private options: BrowserRequirePluginOptions) {}

	apply(compiler: Compiler) {
		const { execute, modules } = this.options;
		compiler.__internal_browser_require = function browserRequire(id: string) {
			// Try to map id to module
			if (typeof modules === "function") {
				const module = modules(id);
				if (module) {
					return module;
				}
			} else if (typeof modules === "object") {
				const module = modules[id];
				if (module) {
					return module;
				}
			}

			// Fallback: resolve in memfs and execute
			if (!execute) {
				throw Error(
					`You should provide 'execute' option if there's no mapping for module '${id}'`
				);
			}

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
			return exports.default ?? module.exports;
		};
	}
}
