import { Mode, RspackOptions } from "@rspack/core";
import { RspackBuildCLIOptions } from "../types";
import { findFileWithSupportedExtensions } from "./loadConfig";
import path from 'node:path';

const defaultEntry = "src/index.ts";
export async function buildConfigWithOptions(
	item: RspackOptions,
	options: RspackBuildCLIOptions,
	isColorSupported: boolean
) {
	const mode = buildConfig_mode(item, options);
	buildConfig_entry(item, options)	
	buildConfig_stats(item, isColorSupported);
	buildConfig_analyze(item, options);
	buildConfig_builtin(item, mode);
	buildConfig_devtool(item, mode);
	buildConfig_watch(item, options);
}

function buildConfig_mode(
	item: RspackOptions,
	options: RspackBuildCLIOptions
): Mode {
	// Respect `process.env.NODE_ENV`
	if (
		!item.mode &&
		process.env &&
		process.env.NODE_ENV &&
		(process.env.NODE_ENV === "development" ||
			process.env.NODE_ENV === "production" ||
			process.env.NODE_ENV === "none")
	) {
		item.mode = process.env.NODE_ENV;
	}

	// user parameters always has highest priority than default mode and config mode
	if (options.mode) {
		item.mode = options.mode as Mode;
	}

	// default value "production"
	if (!item.mode) {
		item.mode = "production";
	}

	return item.mode;
}

function buildConfig_entry(item: RspackOptions, options: RspackBuildCLIOptions) {
	let entry = {};
	if (!item.entry) {
				if (options.entry) {
					entry = {
						main: options.entry.map(x => path.resolve(process.cwd(), x))[0] // Fix me when entry supports array
					};
				} else {
					const defaultEntryBase = path.resolve(process.cwd(), defaultEntry);
					const defaultEntryPath =
						findFileWithSupportedExtensions(defaultEntryBase) ||
						defaultEntryBase + ".js"; // default entry is js
					entry = {
						main: defaultEntryPath
					};
				}
				item.entry = entry;
			}
}

function buildConfig_stats(item: RspackOptions, isColorSupported: boolean) {
	if (typeof item.stats === "undefined") {
		item.stats = { preset: "errors-warnings" };
	} else if (typeof item.stats === "boolean") {
		item.stats = item.stats ? { preset: "normal" } : { preset: "none" };
	} else if (typeof item.stats === "string") {
		item.stats = {
			preset: item.stats as
				| "normal"
				| "none"
				| "verbose"
				| "errors-only"
				| "errors-warnings"
		};
	}

	if (isColorSupported && typeof item.stats.colors === "undefined") {
		item.stats.colors = true;
	}
}

async function buildConfig_analyze(
	item: RspackOptions,
	options: RspackBuildCLIOptions
) {
	if (options.analyze) {
		const { BundleAnalyzerPlugin } = await import("webpack-bundle-analyzer");
		(item.plugins ??= []).push({
			name: "rspack-bundle-analyzer",
			apply(compiler) {
				new BundleAnalyzerPlugin({
					generateStatsFile: true
				}).apply(compiler as any);
			}
		});
	}
}

function buildConfig_watch(item: RspackOptions, options: RspackBuildCLIOptions) {
	// cli --watch overrides the watch config
	if (options.watch) {
		item.watch = options.watch;
	}
}

function buildConfig_builtin(item: RspackOptions, mode: Mode) {
	item.builtins = item.builtins || {};
	if (mode === "development") {
		item.builtins.progress = true;
	}

	// no emit assets when run dev server, it will use node_binding api get file content
	if (typeof item.builtins.noEmitAssets === "undefined") {
		item.builtins.noEmitAssets = false; // @FIXME memory fs currently cause problems for outputFileSystem, so we disable it temporarily
	}

	// When mode is set to 'none', optimization.nodeEnv defaults to false.
	if (mode !== "none") {
		item.builtins.define = {
			// User defined `process.env.NODE_ENV` always has highest priority than default define
			"process.env.NODE_ENV": JSON.stringify(mode),
			...item.builtins.define
		};
	}
}

function buildConfig_devtool(item: RspackOptions, mode: Mode) {
	// false is also a valid value for sourcemap, so don't override it
	if (typeof item.devtool === "undefined") {
		item.devtool =
			mode === "production" ? "source-map" : "cheap-module-source-map";
	}
}
