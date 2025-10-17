import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawModuleFederationManifestPluginOptions
} from "@rspack/binding";
import {
	createBuiltinPlugin,
	RspackBuiltinPlugin
} from "../builtin-plugin/base";
import type { Compiler } from "../Compiler";

const MANIFEST_FILE_NAME = "mf-manifest.json";
const STATS_FILE_NAME = "mf-stats.json";
const LOCAL_BUILD_VERSION = "local";
const JSON_EXT = ".json";

function isPlainObject(value: unknown): value is Record<string, unknown> {
	return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function parseJSON<T>(
	input: string,
	guard: (value: unknown) => value is T
): T | undefined {
	try {
		const parsed: unknown = JSON.parse(input);
		if (guard(parsed)) {
			return parsed;
		}
	} catch {
		// ignore malformed json
	}
	return undefined;
}

function readPKGJson(root?: string): Record<string, string> {
	const base = root ? resolve(root) : process.cwd();
	const pkgPath = join(base, "package.json");
	try {
		const content = readFileSync(pkgPath, "utf-8");
		const parsed = parseJSON(content, isPlainObject);
		if (parsed) {
			const filtered: Record<string, string> = {};
			for (const [key, value] of Object.entries(parsed)) {
				if (typeof value === "string") {
					filtered[key] = value;
				}
			}
			if (Object.keys(filtered).length > 0) {
				return filtered;
			}
		}
	} catch {
		// ignore read/parse errors
	}
	return {};
}

function getBuildInfo(isDev: boolean, root?: string): StatsBuildInfo {
	const rootPath = root || process.cwd();
	const pkg = readPKGJson(rootPath);
	const buildVersion = isDev ? LOCAL_BUILD_VERSION : pkg?.version;

	return {
		buildVersion: process.env.MF_BUILD_VERSION || buildVersion || "UNKNOWN",
		buildName: process.env.MF_BUILD_NAME || pkg?.name || "UNKNOWN"
	};
}

interface StatsBuildInfo {
	buildVersion: string;
	buildName?: string;
}

export type RemoteAliasMap = Record<string, { name: string; entry?: string }>;

export type ManifestExposeOption = {
	path: string;
	name: string;
};

export type ManifestSharedOption = {
	name: string;
	version?: string;
	requiredVersion?: string;
	singleton?: boolean;
};

export type ModuleFederationManifestPluginOptions = {
	name?: string;
	globalName?: string;
	filePath?: string;
	disableAssetsAnalyze?: boolean;
	fileName?: string;
	remoteAliasMap?: RemoteAliasMap;
	exposes?: ManifestExposeOption[];
	shared?: ManifestSharedOption[];
};

function getFileName(manifestOptions: ModuleFederationManifestPluginOptions): {
	statsFileName: string;
	manifestFileName: string;
} {
	if (!manifestOptions) {
		return {
			statsFileName: STATS_FILE_NAME,
			manifestFileName: MANIFEST_FILE_NAME
		};
	}

	const filePath =
		typeof manifestOptions === "boolean" ? "" : manifestOptions.filePath || "";
	const fileName =
		typeof manifestOptions === "boolean" ? "" : manifestOptions.fileName || "";

	const addExt = (name: string): string => {
		if (name.endsWith(JSON_EXT)) {
			return name;
		}
		return `${name}${JSON_EXT}`;
	};
	const insertSuffix = (name: string, suffix: string): string => {
		return name.replace(JSON_EXT, `${suffix}${JSON_EXT}`);
	};
	const manifestFileName = fileName ? addExt(fileName) : MANIFEST_FILE_NAME;
	const statsFileName = fileName
		? insertSuffix(manifestFileName, "-stats")
		: STATS_FILE_NAME;

	return {
		statsFileName: join(filePath, statsFileName),
		manifestFileName: join(filePath, manifestFileName)
	};
}

/**
 * JS-side post-processing plugin: reads mf-manifest.json and mf-stats.json, executes additionalData callback and merges/overwrites manifest.
 * To avoid cross-NAPI callback complexity, this plugin runs at the afterProcessAssets stage to ensure Rust-side MfManifestPlugin has already output its artifacts.
 */
export class ModuleFederationManifestPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ModuleFederationManifestPlugin;
	private opts: ModuleFederationManifestPluginOptions;
	constructor(opts: ModuleFederationManifestPluginOptions) {
		super();
		this.opts = opts;
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const {
			fileName,
			filePath,
			disableAssetsAnalyze,
			remoteAliasMap,
			exposes,
			shared
		} = this.opts;
		const { statsFileName, manifestFileName } = getFileName(this.opts);

		const rawOptions: RawModuleFederationManifestPluginOptions = {
			name: this.opts.name,
			globalName: this.opts.globalName,
			fileName,
			filePath,
			manifestFileName,
			statsFileName,
			disableAssetsAnalyze,
			remoteAliasMap,
			exposes,
			shared,
			buildInfo: getBuildInfo(
				compiler.options.mode === "development",
				compiler.context
			)
		};
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
