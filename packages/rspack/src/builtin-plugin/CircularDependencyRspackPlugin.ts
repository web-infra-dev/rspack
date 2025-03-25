import {
	BuiltinPluginName,
	type JsCompilation,
	type RawCircularDependencyRspackPluginOptions
} from "@rspack/binding";
import type { Module } from "../Module";
import { create } from "./base";

export type CircularDependencyRspackPluginOptions = {
	/**
	 * When `true`, the plugin will emit `ERROR` diagnostics rather than the
	 * default `WARN` level.
	 */
	failOnError?: boolean;
	/**
	 * When `true`, asynchronous imports like `import("some-module")` will not
	 * be considered connections that can create cycles.
	 */
	allowAsyncCycles?: boolean;
	/**
	 * Cycles containing any module name that matches this regex will _not_ be
	 * counted as a cycle.
	 */
	exclude?: RegExp;
	/**
	 * List of dependency connections that should not count for creating cycles.
	 * Connections are represented as `[from, to]`, where each entry is matched
	 * against the _identifier_ for that module in the connection. The
	 * identifier contains the full, unique path for the module, including all
	 * of the loaders that were applied to it and any request parameters.
	 *
	 * When an entry is a String, it is tested as a _substring_ of the
	 * identifier. For example, the entry "components/Button" would match the
	 * module "app/design/components/Button.tsx". When the entry is a RegExp,
	 * it is tested against the entire identifier.
	 */
	ignoredConnections?: Array<[string | RegExp, string | RegExp]>;
	/**
	 * Called once for every detected cycle. Providing this handler overrides the
	 * default behavior of adding diagnostics to the compilation.
	 */
	onDetected?(
		entrypoint: Module,
		modules: string[],
		compilation: JsCompilation
	): void;
	/**
	 * Called once for every detected cycle that was ignored because of a rule,
	 * either from `exclude` or `ignoredConnections`.
	 */
	onIgnored?(
		entrypoint: Module,
		modules: string[],
		compilation: JsCompilation
	): void;
	/**
	 * Called before cycle detection begins.
	 */
	onStart?(compilation: JsCompilation): void;
	/**
	 * Called after cycle detection finishes.
	 */
	onEnd?(compilation: JsCompilation): void;
};

export const CircularDependencyRspackPlugin = create(
	BuiltinPluginName.CircularDependencyRspackPlugin,
	(
		options: CircularDependencyRspackPluginOptions
	): RawCircularDependencyRspackPluginOptions => {
		return {
			allowAsyncCycles: options.allowAsyncCycles,
			failOnError: options.failOnError,
			exclude: options.exclude,
			ignoredConnections: options.ignoredConnections,
			onDetected: options.onDetected,
			onIgnored: options.onIgnored,
			onStart: options.onStart,
			onEnd: options.onEnd
		};
	},
	"compilation"
);
