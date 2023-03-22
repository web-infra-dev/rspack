import path from "path";
import fs from "fs";
import { RspackCLIOptions } from "../types";
import { RspackOptions, MultiRspackOptions } from "@rspack/core";

const supportedExtensions = [".js", ".ts"];
const defaultConfig = "rspack.config";
const defaultEntry = "src/index";

export type LoadedRspackConfig =
	| undefined
	| RspackOptions
	| MultiRspackOptions
	| ((
			env: Record<string, any>,
			argv: Record<string, any>
	  ) => RspackOptions | MultiRspackOptions);

export async function loadRspackConfig(
	options: RspackCLIOptions
): Promise<LoadedRspackConfig> {
	let loadedConfig: LoadedRspackConfig;
	// if we pass config paras
	if (options.config) {
		const resolvedConfigPath = path.resolve(process.cwd(), options.config);
		if (!fs.existsSync(resolvedConfigPath)) {
			throw new Error(`config file "${resolvedConfigPath}" not exists`);
		}
		loadedConfig = await requireWithAdditionalExtension(resolvedConfigPath);
	} else {
		let defaultConfigPath = findFileWithSupportedExtensions(
			path.resolve(process.cwd(), defaultConfig)
		);
		if (defaultConfigPath != null) {
			loadedConfig = await requireWithAdditionalExtension(defaultConfigPath);
		} else {
			let entry: Record<string, string> = {};
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
			loadedConfig = {
				entry
			};
		}
	}
	return loadedConfig;
}

// takes a basePath like `webpack.config`, return `webpack.config.{js,ts}` if
// exists. returns null if none of them exists
function findFileWithSupportedExtensions(basePath: string): string | null {
	for (const extension of supportedExtensions) {
		if (fs.existsSync(basePath + extension)) {
			return basePath + extension;
		}
	}
	return null;
}

let hasRegisteredTS = false;
async function requireWithAdditionalExtension(resolvedPath: string) {
	if (resolvedPath.endsWith("ts") && !hasRegisteredTS) {
		hasRegisteredTS = true;
		let tsNode: any;
		try {
			tsNode = require("ts-node");
		} catch (e) {
			throw new Error("`ts-node` is required to use TypeScript configuration.");
		}
		tsNode.register({ transpileOnly: true });
	}
	let loadedConfig;
	if (resolvedPath.endsWith("ts")) {
		loadedConfig = require(resolvedPath);
	} else {
		// dynamic import can handle both cjs & mjs
		loadedConfig = (await import(resolvedPath)).default;
	}
	return loadedConfig;
}
