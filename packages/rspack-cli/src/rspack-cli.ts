import type { RspackPluginFunction, RspackPluginInstance } from "@rspack/core";
import {
	Compiler,
	MultiCompiler,
	MultiRspackOptions,
	MultiStats,
	rspack,
	RspackOptions,
	Stats
} from "@rspack/core";
import * as rspackCore from "@rspack/core";
import path from "path";
import semver from "semver";
import util from "util";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

import { BuildCommand } from "./commands/build";
import { PreviewCommand } from "./commands/preview";
import { ServeCommand } from "./commands/serve";
import {
	RspackBuildCLIOptions,
	RspackCLIColors,
	RspackCLILogger,
	RspackCLIOptions
} from "./types";
import findConfig from "./utils/findConfig";
import { LoadedRspackConfig, loadRspackConfig } from "./utils/loadConfig";
import { normalizeEnv } from "./utils/options";

type Command = "serve" | "build";

const defaultEntry = "src/index";
export class RspackCLI {
	colors: RspackCLIColors;
	program: yargs.Argv;
	constructor() {
		this.colors = this.createColors();
		this.program = yargs();
	}
	async createCompiler(
		options: RspackBuildCLIOptions,
		rspackCommand: Command,
		callback?: (e: Error | null, res?: Stats | MultiStats) => void
	) {
		process.env.RSPACK_CONFIG_VALIDATE = "loose";
		process.env.WATCHPACK_WATCHER_LIMIT =
			process.env.WATCHPACK_WATCHER_LIMIT || "20";
		let nodeEnv = process?.env?.NODE_ENV;
		let rspackCommandDefaultEnv =
			rspackCommand === "build" ? "production" : "development";
		if (typeof options.nodeEnv === "string") {
			process.env.NODE_ENV = nodeEnv || options.nodeEnv;
		} else {
			process.env.NODE_ENV = nodeEnv || rspackCommandDefaultEnv;
		}
		let config = await this.loadConfig(options);
		config = await this.buildConfig(config, options, rspackCommand);

		const isWatch = Array.isArray(config)
			? (config as MultiRspackOptions).some(i => i.watch)
			: (config as RspackOptions).watch;

		return rspack(config, isWatch ? callback : undefined);
	}
	createColors(useColor?: boolean): RspackCLIColors {
		const { createColors, isColorSupported } = require("colorette");

		let shouldUseColor;

		if (useColor) {
			shouldUseColor = useColor;
		} else {
			shouldUseColor = isColorSupported;
		}

		return {
			...createColors({ useColor: shouldUseColor }),
			isColorSupported: shouldUseColor
		};
	}
	getLogger(): RspackCLILogger {
		return {
			error: val =>
				console.error(`[rspack-cli] ${this.colors.red(util.format(val))}`),
			warn: val => console.warn(`[rspack-cli] ${this.colors.yellow(val)}`),
			info: val => console.info(`[rspack-cli] ${this.colors.cyan(val)}`),
			success: val => console.log(`[rspack-cli] ${this.colors.green(val)}`),
			log: val => console.log(`[rspack-cli] ${val}`),
			raw: val => console.log(val)
		};
	}
	async run(argv: string[]) {
		if (semver.lt(semver.clean(process.version)!, "14.0.0")) {
			this.getLogger().warn(
				`Minimum recommended Node.js version is 14.0.0, current version is ${process.version}`
			);
		}

		this.program.showHelpOnFail(false);
		this.program.usage("[options]");
		this.program.scriptName("rspack");
		this.program.strictCommands(true).strict(true);
		this.program.middleware(normalizeEnv);
		this.registerCommands();
		await this.program.parseAsync(hideBin(argv));
	}
	async registerCommands() {
		const builtinCommands = [
			new BuildCommand(),
			new ServeCommand(),
			new PreviewCommand()
		];
		for (const command of builtinCommands) {
			command.apply(this);
		}
	}
	async buildConfig(
		item: RspackOptions | MultiRspackOptions,
		options: RspackBuildCLIOptions,
		command: Command
	): Promise<RspackOptions | MultiRspackOptions> {
		let commandDefaultEnv: "production" | "development" =
			command === "build" ? "production" : "development";
		let isBuild = command === "build";
		let isServe = command === "serve";
		const internalBuildConfig = async (item: RspackOptions) => {
			if (options.entry) {
				item.entry = {
					main: options.entry.map(x => path.resolve(process.cwd(), x))[0] // Fix me when entry supports array
				};
			} else if (!item.entry) {
				const defaultEntryBase = path.resolve(process.cwd(), defaultEntry);
				const defaultEntryPath =
					findConfig(defaultEntryBase) || defaultEntryBase + ".js"; // default entry is js
				item.entry = {
					main: defaultEntryPath
				};
			}
			// to set output.path
			item.output = item.output || {};
			if (options["output-path"]) {
				item.output.path = path.resolve(process.cwd(), options["output-path"]);
			}
			if (options.analyze) {
				const { BundleAnalyzerPlugin } = await import(
					"webpack-bundle-analyzer"
				);
				(item.plugins ??= []).push({
					name: "rspack-bundle-analyzer",
					apply(compiler) {
						new BundleAnalyzerPlugin({
							generateStatsFile: true
						}).apply(compiler as any);
					}
				});
			}
			if (process.env.RSPACK_PROFILE) {
				const { applyProfile } = await import("./utils/profile.js");
				await applyProfile(process.env.RSPACK_PROFILE, item);
			}
			// cli --watch overrides the watch config
			if (options.watch) {
				item.watch = options.watch;
			}
			// auto set default mode if user config don't set it
			if (!item.mode) {
				item.mode = commandDefaultEnv ?? "none";
			}
			// user parameters always has highest priority than default mode and config mode
			if (options.mode) {
				item.mode = options.mode as RspackOptions["mode"];
			}

			// false is also a valid value for sourcemap, so don't override it
			if (typeof item.devtool === "undefined") {
				item.devtool = isBuild ? "source-map" : "cheap-module-source-map";
			}
			item.builtins = item.builtins || {};
			if (isServe) {
				let installed = (item.plugins ||= []).find(
					item => item instanceof rspackCore.ProgressPlugin
				);
				if (!installed) {
					(item.plugins ??= []).push(new rspackCore.ProgressPlugin());
				}
			}

			// Tells webpack to set process.env.NODE_ENV to a given string value.
			// optimization.nodeEnv uses DefinePlugin unless set to false.
			// optimization.nodeEnv defaults to mode if set, else falls back to 'production'.
			// See doc: https://webpack.js.org/configuration/optimization/#optimizationnodeenv
			// See source: https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/WebpackOptionsApply.js#L563

			// When mode is set to 'none', optimization.nodeEnv defaults to false.
			if (item.mode !== "none") {
				(item.plugins ||= []).push(
					new rspackCore.DefinePlugin({
						// User defined `process.env.NODE_ENV` always has highest priority than default define
						"process.env.NODE_ENV": JSON.stringify(item.mode)
					})
				);
			}

			if (typeof item.stats === "undefined") {
				item.stats = { preset: "errors-warnings", timings: true };
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
			if (
				this.colors.isColorSupported &&
				typeof item.stats.colors === "undefined"
			) {
				item.stats.colors = true;
			}
			return item;
		};

		if (Array.isArray(item)) {
			return Promise.all(item.map(internalBuildConfig));
		} else {
			return internalBuildConfig(item as RspackOptions);
		}
	}

	async loadConfig(
		options: RspackCLIOptions
	): Promise<RspackOptions | MultiRspackOptions> {
		let loadedConfig = (await loadRspackConfig(
			options
		)) as NonNullable<LoadedRspackConfig>;
		if (options.configName) {
			const notFoundConfigNames: string[] = [];

			loadedConfig = options.configName.map((configName: string) => {
				let found: RspackOptions | undefined;

				if (Array.isArray(loadedConfig)) {
					found = loadedConfig.find(options => options.name === configName);
				} else {
					found =
						(loadedConfig as RspackOptions).name === configName
							? (loadedConfig as RspackOptions)
							: undefined;
				}

				if (!found) {
					notFoundConfigNames.push(configName);
				}

				// WARNING: if config is not found, the program will exit
				// so assert here is okay to avoid runtime filtering
				return found!;
			});

			if (notFoundConfigNames.length > 0) {
				this.getLogger().error(
					notFoundConfigNames
						.map(
							configName =>
								`Configuration with the name "${configName}" was not found.`
						)
						.join(" ")
				);
				process.exit(2);
			}
		}

		if (typeof loadedConfig === "function") {
			loadedConfig = loadedConfig(options.argv?.env, options.argv);
			// if return promise we should await its result
			if (
				typeof (loadedConfig as unknown as Promise<unknown>).then === "function"
			) {
				loadedConfig = await loadedConfig;
			}
		}
		return loadedConfig;
	}

	isMultipleCompiler(
		compiler: Compiler | MultiCompiler
	): compiler is MultiCompiler {
		return Boolean((compiler as MultiCompiler).compilers);
	}
	isWatch(compiler: Compiler | MultiCompiler): boolean {
		return Boolean(
			this.isMultipleCompiler(compiler)
				? compiler.compilers.some(compiler => compiler.options.watch)
				: compiler.options.watch
		);
	}
}

export function defineConfig(config: RspackOptions): RspackOptions {
	return config;
}

// Note: use union type will make apply function's `compiler` type to be `any`
export function definePlugin(
	plugin: RspackPluginFunction
): RspackPluginFunction;
export function definePlugin(
	plugin: RspackPluginInstance
): RspackPluginInstance;
export function definePlugin(plugin: any): any {
	return plugin;
}
