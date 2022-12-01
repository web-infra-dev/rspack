import { hideBin } from "yargs/helpers";
import yargs from "yargs";
import util from "util";
import path from "path";
import fs from "fs";
import { RspackCLIColors, RspackCLILogger, RspackCLIOptions } from "./types";
import { BuildCommand } from "./commands/build";
import { ServeCommand } from "./commands/serve";
import { rspack, RspackOptions, createCompiler } from "@rspack/core";
const defaultConfig = "rspack.config.js";
const defaultEntry = "src/index.js";
type Callback<T> = <T>(err: Error, res?: T) => void;
type RspackEnv = "development" | "production";
export class RspackCLI {
	colors: RspackCLIColors;
	program: yargs.Argv<{}>;
	constructor() {
		this.colors = this.createColors();
		this.program = yargs();
	}
	async createCompiler(options: RspackCLIOptions, rspackEnv: RspackEnv) {
		let config = await this.loadConfig(options);
		config = await this.buildConfig(config, options, rspackEnv);
		const compiler = createCompiler(config);
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
		item: any,
		options: RspackCLIOptions,
		rspackEnv: RspackEnv
	) {
		const isEnvProduction = rspackEnv === "production";
		const isEnvDevelopment = rspackEnv === "development";

		if (options.analyze) {
			const { BundleAnalyzerPlugin } = await import("webpack-bundle-analyzer");
			(item.plugins ??= []).push({
				name: "rspack-bundle-analyzer",
				apply(compiler) {
					new BundleAnalyzerPlugin({
						generateStatsFile: true,
						// TODO: delete this once runtime refacted.
						excludeAssets: "runtime.js"
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
			item.mode = options.mode;
		}

		// false is also a valid value for sourcemap, so don't override it
		console.log(item);
		if (typeof item.devtool === "undefined") {
			item.devtool = isEnvProduction ? "source-map" : "cheap-module-source-map";
		}
		console.log("after", item);
		item.builtins = {
			...item.builtins,
			define: item.builtins.define ?? {
				"process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV)
			},
			minify: item.builtins?.minify ?? isEnvProduction
		};
		item.output = {
			...item.output,
			publicPath: item.output?.path ?? "/"
		};
		if (typeof item.stats === "undefined") {
			item.stats = { preset: "normal" };
		} else if (typeof item.stats === "boolean") {
			item.stats = item.stats ? { preset: "normal" } : { preset: "none" };
		} else if (typeof item.stats === "string") {
			item.stats = { preset: item.stats };
		}
		if (this.colors.isColorSupported && !item.stats.colors) {
			item.stats.colors = true;
		}
		return item;
	}
	async loadConfig(options: RspackCLIOptions): Promise<RspackOptions> {
		let loadedConfig: RspackOptions;
		// if we pass config paras
		if (options.config) {
			const resolvedConfigPath = path.resolve(process.cwd(), options.config);
			if (!fs.existsSync(resolvedConfigPath)) {
				throw new Error(`config file "${resolvedConfigPath}" not exists`);
			}
			loadedConfig = require(resolvedConfigPath);
		} else {
			let defaultConfigPath = path.resolve(process.cwd(), defaultConfig);
			if (fs.existsSync(defaultConfigPath)) {
				loadedConfig = require(defaultConfigPath);
			} else {
				let entry: Record<string, string> = {};
				if (options.entry) {
					console.log("entry:", options.entry);
					entry = {
						main: options.entry.map(x => path.resolve(process.cwd(), x))[0] // Fix me when entry supports array
					};
				} else {
					entry = {
						main: path.resolve(process.cwd(), defaultEntry)
					};
				}
				loadedConfig = {
					entry
				};
			}
		}
		return loadedConfig;
	}
}
