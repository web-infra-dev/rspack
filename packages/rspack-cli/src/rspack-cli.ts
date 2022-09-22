import { build } from "@rspack/core";
import { createServer } from "@rspack/dev-server";
import { hideBin } from "yargs/helpers";
import yargs, { boolean } from "yargs";
import util from "util";
import path from "path";
import fs from "fs";
import { RspackCLIColors, RspackCLILogger, RspackCLIOptions } from "./types";
const defaultConfig = "rspack.config.js";
const defaultEntry = "src/index.js";
export class RspackCLI {
	colors: RspackCLIColors;
	constructor() {
		this.colors = this.createColors();
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
		let program = yargs(hideBin(argv));
		const commonOptions = (yargs: yargs.Argv<{}>) => {
			return yargs
				.positional("entry", {
					type: "string",
					array: true,
					describe: "entry"
				})
				.options({
					config: {
						type: "string",
						describe: "config file",
						alias: "c"
					},
					mode: { type: "string", default: "none", describe: "mode" },
					watch: {
						type: "boolean",
						default: false,
						describe: "watch"
					},
					devtool: {
						type: "boolean",
						default: false,
						describe: "devtool"
					}
				});
		};
		program.usage("[options]");
		program.scriptName("rspack");
		program.command(
			["build [entry..]", "$0"],
			"build",
			commonOptions,
			async options => {
				const config = await this.loadConfig(options);
				console.time("build");
				console.log({ config });
				const stats = await build(config);
				console.timeEnd("build");
			}
		);
		program.command(
			"serve [entry..]",
			"serve",
			commonOptions,
			async options => {
				const config = await this.loadConfig(options);

				const server = await createServer(config);
				await server.start();
			}
		);
		await program.parseAsync();
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
