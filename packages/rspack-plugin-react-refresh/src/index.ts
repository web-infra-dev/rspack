import path from "path";
import type { Compiler } from "@rspack/core";
import {
	type NormalizedPluginOptions,
	type PluginOptions,
	normalizeOptions
} from "./options";
import { getAdditionalEntries } from "./utils/getAdditionalEntries";
import getSocketIntegration from "./utils/getSocketIntegration";

export type { PluginOptions };

const reactRefreshPath = require.resolve("../client/reactRefresh.js");
const reactRefreshEntryPath = require.resolve("../client/reactRefreshEntry.js");
const refreshUtilsPath = require.resolve("../client/refreshUtils.js");
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
	options: NormalizedPluginOptions;

	static deprecated_runtimePaths: string[];

	constructor(options: PluginOptions = {}) {
		this.options = normalizeOptions(options);
	}

	apply(compiler: Compiler) {
		if (
			// Webpack do not set process.env.NODE_ENV, so we need to check for mode.
			// Ref: https://github.com/webpack/webpack/issues/7074
			(compiler.options.mode !== "development" ||
				// We also check for production process.env.NODE_ENV,
				// in case it was set and mode is non-development (e.g. 'none')
				(process.env.NODE_ENV && process.env.NODE_ENV === "production")) &&
			!this.options.forceEnable
		) {
			return;
		}
		const addEntries = getAdditionalEntries({
			devServer: compiler.options.devServer,
			options: this.options
		});

		addEntries.prependEntries.forEach(entry => {
			new compiler.webpack.EntryPlugin(compiler.context, entry, {
				name: undefined
			}).apply(compiler);
		})
		addEntries.overlayEntries.forEach(entry => {
			new compiler.webpack.EntryPlugin(compiler.context, entry, {
				name: undefined
			}).apply(compiler);
		})
		new compiler.webpack.ProvidePlugin({
			$ReactRefreshRuntime$: reactRefreshPath
		}).apply(compiler);

		compiler.options.module.rules.unshift({
			include: this.options.include!,
			exclude: {
				or: [this.options.exclude!, [...runtimePaths]].filter(Boolean)
			},
			use: "builtin:react-refresh-loader"
		});

		const definedModules: any = {
			// For Multiple Instance Mode
			__react_refresh_library__: JSON.stringify(
				compiler.webpack.Template.toIdentifier(
					this.options.library ||
					compiler.options.output.uniqueName ||
					compiler.options.output.library
				)
			)
		};
		const providedModules: any = {
			__react_refresh_utils__: refreshUtilsPath
		};
		if (this.options.overlay === false) {
			// Stub errorOverlay module so their calls can be erased
			definedModules.__react_refresh_error_overlay__ = false;
			definedModules.__react_refresh_socket__ = false;
		} else {
			if (this.options.overlay.module) {
				providedModules.__react_refresh_error_overlay__ = require.resolve(
					this.options.overlay.module
				);
			}

			if (this.options.overlay.sockIntegration) {
				providedModules.__react_refresh_socket__ = getSocketIntegration(
					this.options.overlay.sockIntegration
				);
			}
		}
		new compiler.webpack.DefinePlugin(definedModules).apply(compiler);
		new compiler.webpack.ProvidePlugin(providedModules).apply(compiler);

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
