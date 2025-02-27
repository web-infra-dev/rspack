import fs from "node:fs";
import path from "node:path";
import type { MultiRspackOptions, RspackOptions } from "@rspack/core";
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

export async function loadRspackConfig(
	options: RspackCLIOptions,
	cwd = process.cwd()
): Promise<LoadedRspackConfig> {
	if (options.config) {
		const configPath = path.resolve(cwd, options.config);
		if (!fs.existsSync(configPath)) {
			throw new Error(`config file "${configPath}" not found.`);
		}
		if (isTsFile(configPath) && options.configLoader === "register") {
			await registerLoader(configPath);
		}
		return crossImport(configPath, cwd);
	}

	const defaultConfig = findConfig(path.resolve(cwd, DEFAULT_CONFIG_NAME));
	if (defaultConfig) {
		if (isTsFile(defaultConfig) && options.configLoader === "register") {
			await registerLoader(defaultConfig);
		}
		return crossImport(defaultConfig, cwd);
	}
	return {};
}
