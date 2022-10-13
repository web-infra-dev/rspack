import { hideBin } from "yargs/helpers";
import yargs from "yargs";
import util from "util";
import path from "path";
import fs from "fs";
import { RspackCLIColors, RspackCLILogger, RspackCLIOptions } from "./types";
import { BuildCommand } from "./commands/build";
import { ServeCommand } from "./commands/serve";
import { rspack, RspackOptions } from "@rspack/core";

const defaultConfig = "rspack.config.js";
const defaultEntry = "src/index.js";

export class RspackCLI {
	colors: RspackCLIColors;
	program: yargs.Argv<{}>;
	constructor() {
		this.colors = this.createColors();
		this.program = yargs();
	}
	async createCompiler(options: RspackCLIOptions) {
		let config = await this.loadConfig(options);
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
		this.registerCommands();
		await this.program.parseAsync(hideBin(argv));
	}
	async registerCommands() {
		const builtinCommands = [new BuildCommand(), new ServeCommand()];
		for (const command of builtinCommands) {
			command.apply(this);
		}
	}
	async loadConfig(options: RspackCLIOptions) {
		let loadedConfig;
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
