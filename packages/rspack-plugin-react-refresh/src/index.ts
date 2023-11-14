import path from "path";
import type { Compiler } from "@rspack/core";
import { DefinePlugin } from "@rspack/core";
import { normalizeOptions, type PluginOptions } from "./options";

export type { PluginOptions };

const reactRefreshPath = require.resolve("../client/reactRefresh.js");
const reactRefreshEntryPath = require.resolve("../client/reactRefreshEntry.js");

const refreshUtilsPath = require.resolve(
	"@pmmmwh/react-refresh-webpack-plugin/lib/runtime/RefreshUtils",
	{
		paths: [reactRefreshPath]
	}
);
const refreshRuntimeDirPath = path.dirname(
	require.resolve("react-refresh", {
		paths: [reactRefreshPath]
	})
);
const runtimePaths = [
	reactRefreshEntryPath,
	reactRefreshPath,
	refreshUtilsPath,
	refreshRuntimeDirPath
];

/**
 * @typedef {Object} Options
 * @property {(string | RegExp | (string | RegExp)[] | null)=} include included resourcePath for loader
 * @property {(string | RegExp | (string | RegExp)[] | null)=} exclude excluded resourcePath for loader
 */
class ReactRefreshRspackPlugin {
	options: PluginOptions;

	static deprecated_runtimePaths: string[];

	constructor(options: PluginOptions = {}) {
		this.options = normalizeOptions(options);
	}

	apply(compiler: Compiler) {
		new compiler.webpack.EntryPlugin(compiler.context, reactRefreshEntryPath, {
			name: undefined
		}).apply(compiler);
		new compiler.webpack.ProvidePlugin({
			$ReactRefreshRuntime$: reactRefreshPath
		}).apply(compiler);

		compiler.options.module.rules.unshift({
			// @ts-expect-error
			include: this.options.include,
			// @ts-expect-error
			exclude: {
				or: [this.options.exclude, [...runtimePaths]].filter(Boolean)
			},
			use: "builtin:react-refresh-loader"
		});

		const definedModules = {
			// For Mutiple Instance Mode
			__react_refresh_library__: JSON.stringify(
				compiler.webpack.Template.toIdentifier(
					this.options.library ||
						compiler.options.output.uniqueName ||
						compiler.options.output.library
				)
			)
		};
		new DefinePlugin(definedModules).apply(compiler);

		const refreshPath = path.dirname(require.resolve("react-refresh"));
		compiler.options.resolve.alias = {
			"react-refresh": refreshPath,
			...compiler.options.resolve.alias
		};
	}
}

ReactRefreshRspackPlugin.deprecated_runtimePaths = runtimePaths;

// @ts-expect-error output module.exports
export = ReactRefreshRspackPlugin;
