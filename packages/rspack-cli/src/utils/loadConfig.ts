import path from "path";
import fs from "fs";
import { RspackCLIOptions } from "../types";
import { RspackOptions, MultiRspackOptions } from "@rspack/core";

const defaultConfig = "rspack.config.js";
const defaultEntry = "src/index.js";

export type LoadedRspackConfig =
	| undefined
	| RspackOptions
	| MultiRspackOptions
	| ((
			env: Record<string, any>,
			argv: Record<string, any>
	  ) => RspackOptions | MultiRspackOptions);

export function loadRspackConfig(
	options: RspackCLIOptions
): LoadedRspackConfig {
	let loadedConfig: LoadedRspackConfig;
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
