import path from "path";
import fs from "fs";
import { RspackCLIOptions } from "../types";
import { RspackOptions, MultiRspackOptions } from "@rspack/core";
import findExtFile from "./findExtFile";
import jiti from "jiti";
import { transform } from "sucrase";

const DEFAULT_CONFIG_NAME = "rspack.config" as const;

// Use it to load configuration files from https://github.com/unjs/jiti.
const jitiLoad = (() => {
	return jiti(__filename, {
		interopDefault: true,
		cache: true,
		transform: options => {
			return transform(options.source, {
				transforms: ["typescript", "imports"]
			});
		}
	});
})();

export type LoadedRspackConfig =
	| undefined
	| RspackOptions
	| MultiRspackOptions
	| ((
			env: Record<string, any>,
			argv: Record<string, any>
	  ) => RspackOptions | MultiRspackOptions);

export async function loadRspackConfig(
	options: RspackCLIOptions,
	cwd = process.cwd()
): Promise<LoadedRspackConfig> {
	if (options.config) {
		const configPath = path.resolve(cwd, options.config);
		if (!fs.existsSync(configPath)) {
			throw new Error(`config file "${configPath}" not found.`);
		}
		return jitiLoad(configPath);
	} else {
		const defaultConfig = findExtFile(path.resolve(cwd, DEFAULT_CONFIG_NAME));
		if (defaultConfig) {
			return jitiLoad(defaultConfig);
		} else {
			return {};
		}
	}
}
