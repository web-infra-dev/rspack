import fs from "node:fs";
import path from "node:path";
import {
	util,
	type MultiRspackOptions,
	type RspackOptions
} from "@rspack/core";
import type { RspackCLIOptions } from "../types";
import { crossImport } from "./crossImport";
import findConfig from "./findConfig";
import isEsmFile from "./isEsmFile";
import isTsFile from "./isTsFile";

interface RechoirError extends Error {
	failures: RechoirError[];
	error: Error;
}

const DEFAULT_CONFIG_NAME = "rspack.config" as const;

const registerLoader = async (configPath: string) => {
	const ext = path.extname(configPath);
	// TODO implement good `.mts` support after https://github.com/gulpjs/rechoir/issues/43
	// For ESM and `.mts` you need to use: 'NODE_OPTIONS="--loader ts-node/esm" rspack build --config ./rspack.config.mts'
	if (isEsmFile(configPath) && isTsFile(configPath)) {
		return;
	}

	const { default: interpret } = await import("interpret");
	const extensions = Object.fromEntries(
		Object.entries(interpret.extensions).filter(([key]) => key === ext)
	);
	if (Object.keys(extensions).length === 0) {
		throw new Error(`config file "${configPath}" is not supported.`);
	}

	try {
		const { default: rechoir } = await import("rechoir");
		rechoir.prepare(extensions, configPath);
	} catch (error) {
		const failures = (error as RechoirError)?.failures;
		if (failures) {
			const messages = failures.map(failure => failure.error.message);
			throw new Error(`${messages.join("\n")}`);
		}
		throw error;
	}
};

export type LoadedRspackConfig =
	| undefined
	| RspackOptions
	| MultiRspackOptions
	| ((
			env: Record<string, any>,
			argv?: Record<string, any>
	  ) => RspackOptions | MultiRspackOptions);

/**
 * Loads and merges configurations from the 'extends' property
 * @param config The configuration object that may contain an 'extends' property
 * @param configPath The path to the configuration file
 * @param cwd The current working directory
 * @param options CLI options
 * @returns The merged configuration
 */
export async function loadExtendedConfig(
	config: RspackOptions | MultiRspackOptions,
	configPath: string,
	cwd: string,
	options: RspackCLIOptions
): Promise<RspackOptions | MultiRspackOptions> {
	// If there's no extends property, return the config as is
	if (!("extends" in config) || !config.extends) {
		return config;
	}

	// Convert extends to an array if it's a string
	const extendsList = Array.isArray(config.extends)
		? config.extends
		: [config.extends];

	// Remove the extends property to avoid infinite recursion
	const { extends: _, ...configWithoutExtends } = config as RspackOptions;

	// The base directory for resolving relative paths is the directory of the config file
	const baseDir = path.dirname(configPath);

	// Load and merge configurations from right to left
	let resultConfig = configWithoutExtends;

	for (const extendPath of extendsList) {
		let resolvedPath: string;

		// Check if it's a node module or a relative path
		if (
			extendPath.startsWith(".") ||
			extendPath.startsWith("/") ||
			extendPath.includes(":\\")
		) {
			// It's a relative or absolute path
			resolvedPath = path.resolve(baseDir, extendPath);

			// If the path doesn't have an extension, try to find a matching config file
			if (!path.extname(resolvedPath)) {
				const foundConfig = findConfig(resolvedPath);
				if (foundConfig) {
					resolvedPath = foundConfig;
				} else {
					throw new Error(
						`Extended configuration file "${resolvedPath}" not found.`
					);
				}
			}
		} else {
			// It's a node module
			try {
				resolvedPath = require.resolve(extendPath, { paths: [baseDir, cwd] });
			} catch (error) {
				throw new Error(`Cannot find module '${extendPath}' to extend from.`);
			}
		}

		// Check if the file exists
		if (!fs.existsSync(resolvedPath)) {
			throw new Error(
				`Extended configuration file "${resolvedPath}" not found.`
			);
		}

		// Register loader for TypeScript files
		if (isTsFile(resolvedPath) && options.configLoader === "register") {
			await registerLoader(resolvedPath);
		}

		// Load the extended configuration
		let extendedConfig = await crossImport(resolvedPath, cwd);

		// If the extended config is a function, execute it
		if (typeof extendedConfig === "function") {
			extendedConfig = extendedConfig(options.argv?.env, options.argv);
			// if return promise we should await its result
			if (
				typeof (extendedConfig as unknown as Promise<unknown>).then ===
				"function"
			) {
				extendedConfig = await extendedConfig;
			}
		}

		// Recursively load extended configurations from the extended config
		extendedConfig = await loadExtendedConfig(
			extendedConfig,
			resolvedPath,
			cwd,
			options
		);

		// Merge the configurations
		resultConfig = util.cleverMerge(extendedConfig, resultConfig);
	}

	return resultConfig;
}

export async function loadRspackConfig(
	options: RspackCLIOptions,
	cwd = process.cwd()
): Promise<LoadedRspackConfig> {
	let configPath: string | undefined;
	let loadedConfig: LoadedRspackConfig;

	if (options.config) {
		configPath = path.resolve(cwd, options.config);
		if (!fs.existsSync(configPath)) {
			throw new Error(`config file "${configPath}" not found.`);
		}
		if (isTsFile(configPath) && options.configLoader === "register") {
			await registerLoader(configPath);
		}
		loadedConfig = await crossImport(configPath, cwd);
	} else {
		const defaultConfig = findConfig(path.resolve(cwd, DEFAULT_CONFIG_NAME));
		if (defaultConfig) {
			configPath = defaultConfig;
			if (isTsFile(defaultConfig) && options.configLoader === "register") {
				await registerLoader(defaultConfig);
			}
			loadedConfig = await crossImport(defaultConfig, cwd);
		} else {
			return {};
		}
	}

	// Handle extends property if the loaded config is not a function
	if (typeof loadedConfig !== "function" && configPath) {
		loadedConfig = await loadExtendedConfig(
			loadedConfig as RspackOptions | MultiRspackOptions,
			configPath,
			cwd,
			options
		);
	}

	return loadedConfig;
}
