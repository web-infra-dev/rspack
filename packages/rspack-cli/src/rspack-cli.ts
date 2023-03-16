import { hideBin } from "yargs/helpers";
import yargs from "yargs";
import util from "util";
import { RspackCLIColors, RspackCLILogger, RspackCLIOptions } from "./types";
import { BuildCommand } from "./commands/build";
import { ServeCommand } from "./commands/serve";
import {
	RspackOptions,
	MultiCompiler,
	Compiler,
	rspack,
	MultiRspackOptions
} from "@rspack/core";
import { normalizeEnv } from "./utils/options";
import { loadRspackConfig } from "./utils/loadConfig";
import { Mode } from "@rspack/core/src/config";

type RspackEnv = "development" | "production";
export class RspackCLI {
	colors: RspackCLIColors;
	program: yargs.Argv<{}>;
	constructor() {
		this.colors = this.createColors();
		this.program = yargs();
	}
	async createCompiler(
		options: RspackCLIOptions,
		rspackEnv: RspackEnv
	): Promise<Compiler | MultiCompiler> {
		process.env.RSPACK_CONFIG_VALIDATE = "loose";
		let nodeEnv = process?.env?.NODE_ENV;
		if (typeof options.nodeEnv === "string") {
			process.env.NODE_ENV = nodeEnv || options.nodeEnv;
		} else {
			process.env.NODE_ENV = nodeEnv || rspackEnv;
		}
		let config = await this.loadConfig(options);
		config = await this.buildConfig(config, options, rspackEnv);
		// @ts-ignore
		const compiler = rspack(config);
		return compiler;
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
		this.program.usage("[options]");
		this.program.scriptName("rspack");
		this.program.middleware(normalizeEnv);
		this.registerCommands();
		await this.program.parseAsync(hideBin(argv));
	}
	async registerCommands() {
		const builtinCommands = [new BuildCommand(), new ServeCommand()];
		for (const command of builtinCommands) {
			command.apply(this);
		}
	}
	async buildConfig(
		item: RspackOptions | MultiRspackOptions,
		options: RspackCLIOptions,
		rspackEnv: RspackEnv
	): Promise<RspackOptions | MultiRspackOptions> {
		const internalBuildConfig = async (item: RspackOptions) => {
			const isEnvProduction = rspackEnv === "production";
			const isEnvDevelopment = rspackEnv === "development";

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
			// auto set default mode if user config don't set it
			if (!item.mode) {
				item.mode = rspackEnv ?? "none";
			}
			// user parameters always has highest priority than default mode and config mode
			if (options.mode) {
				item.mode = options.mode as Mode;
			}

			// false is also a valid value for sourcemap, so don't override it
			if (typeof item.devtool === "undefined") {
				item.devtool = isEnvProduction
					? "source-map"
					: "cheap-module-source-map";
			}
			item.builtins = item.builtins || {};
			if (isEnvDevelopment) {
				item.builtins.progress = true;
			}

			// no emit assets when run dev server, it will use node_binding api get file content
			if (typeof item.builtins.noEmitAssets === "undefined") {
				item.builtins.noEmitAssets = false; // @FIXME memory fs currently cause problems for outputFileSystem, so we disable it temporarily
			}

			// Tells webpack to set process.env.NODE_ENV to a given string value.
			// optimization.nodeEnv uses DefinePlugin unless set to false.
			// optimization.nodeEnv defaults to mode if set, else falls back to 'production'.
			// See doc: https://webpack.js.org/configuration/optimization/#optimizationnodeenv
			// See source: https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/WebpackOptionsApply.js#L563

			// When mode is set to 'none', optimization.nodeEnv defaults to false.
			if (item.mode !== "none") {
				item.builtins.define = {
					// User defined `process.env.NODE_ENV` always has highest priority than default define
					"process.env.NODE_ENV": JSON.stringify(item.mode),
					...item.builtins.define
				};
			}

			if (typeof item.stats === "undefined") {
				item.stats = { preset: "normal" };
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
		let loadedConfig = loadRspackConfig(options);
		if (options.configName) {
			const notFoundConfigNames: string[] = [];

			// @ts-ignore
			loadedConfig = options.configName.map((configName: string) => {
				let found: RspackOptions | MultiRspackOptions | undefined;

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

				return found;
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
}

export function defineConfig(config: RspackOptions): RspackOptions {
	return config;
}
