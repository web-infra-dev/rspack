import path from "path";
import fs from "fs";
import { RspackCLIOptions } from "../types";
import { RspackOptions, MultiRspackOptions } from "@rspack/core";
import findExtFile from "./findExtFile";
import { rspack } from "@rspack/core";
import isESM from "./isESM";
import { build } from "esbuild";

const DEFAULT_CONFIG_NAME = "rspack.config" as const;

const buildConfig = async (options: {
	entry: string;
	output: string;
	type: "esm" | "cjs";
}) => {
	const { entry, output, type } = options;

	await build({
		absWorkingDir: process.cwd(),
		bundle: true,
		entryPoints: [entry],
		outfile: output,
		format: type,
		target: ["node14.18", "node16"], // Ignore the user's ts tsconfig
		write: true,
		sourcemap: "inline",
		platform: "node",
		external: ["@rspack/core"]
	});

	//  TODO: Bootstrap with rspack?
	//  I don't know how to configure to output the correct esm/cjs file.
	//  The import/require out file is always wrong in importConfig.
	//
	// 	const compiler = rspack({
	// 		entry: entry,
	// 		mode: "none",
	// 		target: "node",
	// 		experiments: {
	// 			outputModule: type === "esm"
	// 		},
	// 		output: {
	// 			path: path.dirname(output),
	// 			filename: path.basename(output),
	// 			chunkFormat: type === "esm" ? "module" : "commonjs",
	// 			module: type === "esm",
	// 			clean: false,
	// 			library: {
	// 				type: type
	// 			}
	// 		}
	// 	});
	// 	return new Promise((resolve, reject) =>
	// 		compiler.build(error => (error ? reject(error) : resolve(output)))
	// 	);
	// };
};

const importConfig = async (
	configPath: string
): Promise<LoadedRspackConfig> => {
	const esm = await isESM(configPath);

	const type = esm ? "esm" : "cjs";

	const tempPath = `${configPath}.timestamp-${Date.now()}-${Math.random()
		.toString(16)
		.slice(2)}${type === "esm" ? ".mjs" : ".cjs"}`;

	try {
		await buildConfig({
			entry: configPath,
			output: tempPath,
			type
		});
		const module = await (esm ? import(tempPath) : require(tempPath));
		const config = Reflect.has(module, "default") ? module.default : module;
		return config;
	} catch (error) {
		throw error;
	} finally {
		fs.unlinkSync(tempPath);
	}
};

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
	if (options.config) {
		const configPath = path.resolve(process.cwd(), options.config);
		if (!fs.existsSync(configPath)) {
			throw new Error(`config file "${configPath}" not found.`);
		}
		return importConfig(configPath);
	} else {
		const defaultConfig = findExtFile(
			path.resolve(process.cwd(), DEFAULT_CONFIG_NAME)
		);
		if (defaultConfig) {
			return importConfig(defaultConfig);
		} else {
			return {};
		}
	}
}
