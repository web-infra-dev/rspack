import path from "node:path";
import util from "node:util";
import type { RspackPluginFunction, RspackPluginInstance } from "@rspack/core";
import {
	type Compiler,
	type MultiCompiler,
	type MultiRspackOptions,
	type MultiStats,
	type RspackOptions,
	type Stats,
	ValidationError,
	rspack
} from "@rspack/core";
import * as rspackCore from "@rspack/core";
import { createColors, isColorSupported } from "colorette";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { BuildCommand } from "./commands/build";
import { PreviewCommand } from "./commands/preview";
import { ServeCommand } from "./commands/serve";
import type {
	RspackBuildCLIOptions,
	RspackCLIColors,
	RspackCLILogger,
	RspackCLIOptions
} from "./types";
import { type LoadedRspackConfig, loadRspackConfig } from "./utils/loadConfig";
import { normalizeEnv } from "./utils/options";

type Command = "serve" | "build";

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
		process.env.RSPACK_CONFIG_VALIDATE ??= "loose";
		process.env.WATCHPACK_WATCHER_LIMIT =
			process.env.WATCHPACK_WATCHER_LIMIT || "20";

		let config = await this.loadConfig(options);
		config = await this.buildConfig(config, options, rspackCommand);

		const isWatch = Array.isArray(config)
			? (config as MultiRspackOptions).some(i => i.watch)
			: (config as RspackOptions).watch;

		let compiler: MultiCompiler | Compiler | null;
		try {
			compiler = rspack(config, isWatch ? callback : undefined);
		} catch (e) {
			// Aligned with webpack-cli
			// See: https://github.com/webpack/webpack-cli/blob/eea6adf7d34dfbfd3b5b784ece4a4664834f5a6a/packages/webpack-cli/src/webpack-cli.ts#L2394
			if (e instanceof ValidationError) {
				this.getLogger().error(e.message);
				process.exit(2);
			} else if (e instanceof Error) {
				if (typeof callback === "function") {
					callback(e);
				} else {
					this.getLogger().error(e);
				}
				return null;
			}
			throw e;
		}
		return compiler;
	}
	createColors(useColor?: boolean): RspackCLIColors {
		const shouldUseColor = useColor || isColorSupported;
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
		const isBuild = command === "build";
		const isServe = command === "serve";

		const internalBuildConfig = async (item: RspackOptions) => {
			if (options.entry) {
				item.entry = {
					main: options.entry.map(x => path.resolve(process.cwd(), x))[0] // Fix me when entry supports array
				};
			}
			// to set output.path
			item.output = item.output || {};
			if (options.outputPath) {
				item.output.path = path.resolve(process.cwd(), options.outputPath);
			}
			if (options.analyze) {
				const { BundleAnalyzerPlugin } = await import(
					"webpack-bundle-analyzer"
				);
				(item.plugins ??= []).push({
					name: "rspack-bundle-analyzer",
					apply(compiler: any) {
						new BundleAnalyzerPlugin({
							generateStatsFile: true
						}).apply(compiler);
					}
				});
			}
			if (options.profile) {
				item.profile = true;
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
				item.mode = isBuild ? "production" : "development";
			}
			// user parameters always has highest priority than default mode and config mode
			if (options.mode) {
				item.mode = options.mode as RspackOptions["mode"];
			}

			// false is also a valid value for sourcemap, so don't override it
			if (typeof item.devtool === "undefined") {
				item.devtool = isBuild ? "source-map" : "cheap-module-source-map";
			}
			if (isServe) {
				const installed = (item.plugins ||= []).find(
					item => item instanceof rspackCore.ProgressPlugin
				);
				if (!installed) {
					(item.plugins ??= []).push(new rspackCore.ProgressPlugin());
				}
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
		}
		return internalBuildConfig(item as RspackOptions);
	}

	async loadConfig(
		options: RspackCLIOptions
	): Promise<RspackOptions | MultiRspackOptions> {
		let loadedConfig = (await loadRspackConfig(
			options
		)) as NonNullable<LoadedRspackConfig>;

		if (typeof loadedConfig === "function") {
			loadedConfig = loadedConfig(options.argv?.env, options.argv);
			// if return promise we should await its result
			if (
				typeof (loadedConfig as unknown as Promise<unknown>).then === "function"
			) {
				loadedConfig = await loadedConfig;
			}
		}

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

export type RspackConfigFn = (
	env: Record<string, any>,
	argv: Record<string, any>
) => RspackOptions | MultiRspackOptions;

export type RspackConfigAsyncFn = (
	env: Record<string, any>,
	argv: Record<string, any>
) => Promise<RspackOptions | MultiRspackOptions>;

export type RspackConfigExport =
	| RspackOptions
	| MultiRspackOptions
	| RspackConfigFn
	| RspackConfigAsyncFn;

/**
 * This function helps you to autocomplete configuration types.
 * It accepts a Rspack config object, or a function that returns a config.
 */
export function defineConfig(config: RspackOptions): RspackOptions;
export function defineConfig(config: MultiRspackOptions): MultiRspackOptions;
export function defineConfig(config: RspackConfigFn): RspackConfigFn;
export function defineConfig(config: RspackConfigAsyncFn): RspackConfigAsyncFn;
export function defineConfig(config: RspackConfigExport): RspackConfigExport;
export function defineConfig(config: RspackConfigExport) {
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
