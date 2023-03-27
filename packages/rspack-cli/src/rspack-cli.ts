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
	MultiRspackOptions,
	Stats,
	MultiStats
} from "@rspack/core";
import { normalizeEnv } from "./utils/options";
import { loadRspackConfig } from "./utils/loadConfig";
import { RspackPluginInstance, RspackPluginFunction } from "@rspack/core";
import { buildConfigWithOptions } from "./utils/buildConfig";

export class RspackCLI {
	colors: RspackCLIColors;
	program: yargs.Argv<{}>;
	constructor() {
		this.colors = this.createColors();
		this.program = yargs();
	}
	async createCompiler(
		options: RspackCLIOptions,
		callback?: (e: Error, res?: Stats | MultiStats) => void
	): Promise<Compiler | MultiCompiler> {
		process.env.RSPACK_CONFIG_VALIDATE = "loose";
		if (typeof options.nodeEnv === "string") {
			process.env.NODE_ENV = options.nodeEnv;
		}

		let config = await this.loadConfig(options);
		config = await this.buildConfig(config, options);

		const compiler = rspack(config, callback);
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
		options: RspackCLIOptions
	): Promise<RspackOptions | MultiRspackOptions> {
		const internalBuildConfig = async (item: RspackOptions) => {
			buildConfigWithOptions(item, options, this.colors.isColorSupported);
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
		let loadedConfig = await loadRspackConfig(options);
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
