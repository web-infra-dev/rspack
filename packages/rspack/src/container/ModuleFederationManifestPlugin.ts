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
import {
	normalizeSharedOptions,
	type SharedConfig
} from "../sharing/SharePlugin";
import { isRequiredVersion } from "../sharing/utils";
import {
	getRemoteInfos,
	type ModuleFederationPluginOptions
} from "./ModuleFederationPlugin";
import { parseOptions } from "./options";

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

function getBuildInfo(
	isDev: boolean,
	compiler: Compiler,
	mfConfig: ModuleFederationPluginOptions
): StatsBuildInfo {
	const rootPath = compiler.options.context || process.cwd();
	const pkg = readPKGJson(rootPath);
	const buildVersion = isDev ? LOCAL_BUILD_VERSION : pkg?.version;

	const statsBuildInfo: StatsBuildInfo = {
		buildVersion: process.env.MF_BUILD_VERSION || buildVersion || "UNKNOWN",
		buildName: process.env.MF_BUILD_NAME || pkg?.name || "UNKNOWN"
	};

	const normalizedShared = normalizeSharedOptions(mfConfig.shared || {});
	const enableTreeshake = Object.values(normalizedShared).some(
		config => config[1].treeshake
	);
	if (enableTreeshake) {
		statsBuildInfo.target = Array.isArray(compiler.options.target)
			? compiler.options.target
			: [];
		statsBuildInfo.plugins = mfConfig.treeshakeSharedExcludedPlugins || [];
	}

	return statsBuildInfo;
}

interface StatsBuildInfo {
	buildVersion: string;
	buildName?: string;
	// only appear when enable treeshake
	target?: string[];
	plugins?: string[];
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

type InternalManifestPluginOptions = {
	name?: string;
	globalName?: string;
	filePath?: string;
	disableAssetsAnalyze?: boolean;
	fileName?: string;
	remoteAliasMap?: RemoteAliasMap;
	exposes?: ManifestExposeOption[];
	shared?: ManifestSharedOption[];
};

export type ModuleFederationManifestPluginOptions =
	| boolean
	| Pick<
			InternalManifestPluginOptions,
			"disableAssetsAnalyze" | "filePath" | "fileName"
	  >;

export function getFileName(
	manifestOptions: ModuleFederationManifestPluginOptions
): {
	statsFileName: string;
	manifestFileName: string;
} {
	if (!manifestOptions) {
		return {
			statsFileName: "",
			manifestFileName: ""
		};
	}

	if (typeof manifestOptions === "boolean") {
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

function resolveLibraryGlobalName(
	library: ModuleFederationPluginOptions["library"]
): string | undefined {
	if (!library) {
		return undefined;
	}
	const libName = library.name;
	if (!libName) {
		return undefined;
	}
	if (typeof libName === "string") {
		return libName;
	}
	if (Array.isArray(libName)) {
		return libName[0];
	}
	if (typeof libName === "object") {
		return libName.root?.[0] ?? libName.amd ?? libName.commonjs ?? undefined;
	}
	return undefined;
}

function collectManifestExposes(
	exposes: ModuleFederationPluginOptions["exposes"]
): ManifestExposeOption[] | undefined {
	if (!exposes) return undefined;
	type NormalizedExpose = { import: string[]; name?: string };
	type ExposesConfigInput = { import: string | string[]; name?: string };
	const parsed = parseOptions<ExposesConfigInput, NormalizedExpose>(
		exposes,
		value => ({
			import: Array.isArray(value) ? value : [value],
			name: undefined
		}),
		value => ({
			import: Array.isArray(value.import) ? value.import : [value.import],
			name: value.name ?? undefined
		})
	);
	const result = parsed.map(([exposeKey, info]) => {
		const exposeName = info.name ?? exposeKey.replace(/^\.\//, "");
		return {
			path: exposeKey,
			name: exposeName
		};
	});
	return result.length > 0 ? result : undefined;
}

function collectManifestShared(
	shared: ModuleFederationPluginOptions["shared"]
): ManifestSharedOption[] | undefined {
	if (!shared) return undefined;
	const parsed = parseOptions<SharedConfig, SharedConfig>(
		shared,
		(item, key) => {
			if (typeof item !== "string") {
				throw new Error("Unexpected array in shared");
			}
			return item === key || !isRequiredVersion(item)
				? { import: item }
				: { import: key, requiredVersion: item };
		},
		item => item
	);
	const result = parsed.map(([key, config]) => {
		const name = config.shareKey || key;
		const version =
			typeof config.version === "string" ? config.version : undefined;
		const requiredVersion =
			typeof config.requiredVersion === "string"
				? config.requiredVersion
				: undefined;
		return {
			name,
			version,
			requiredVersion,
			singleton: config.singleton
		};
	});
	return result.length > 0 ? result : undefined;
}

function normalizeManifestOptions(mfConfig: ModuleFederationPluginOptions) {
	const manifestOptions: InternalManifestPluginOptions =
		mfConfig.manifest === true ? {} : { ...mfConfig.manifest };
	const containerName = mfConfig.name;
	const globalName =
		resolveLibraryGlobalName(mfConfig.library) ?? containerName;
	const remoteAliasMap: RemoteAliasMap = Object.entries(
		getRemoteInfos(mfConfig)
	).reduce<RemoteAliasMap>((sum, cur) => {
		if (cur[1].length > 1) {
			// no support multiple remotes
			return sum;
		}
		const remoteInfo = cur[1][0];
		const { entry, alias, name } = remoteInfo;
		if (entry && name) {
			sum[alias] = {
				name,
				entry
			};
		}
		return sum;
	}, {});

	const manifestExposes = collectManifestExposes(mfConfig.exposes);
	if (manifestOptions.exposes === undefined && manifestExposes) {
		manifestOptions.exposes = manifestExposes;
	}
	const manifestShared = collectManifestShared(mfConfig.shared);
	if (manifestOptions.shared === undefined && manifestShared) {
		manifestOptions.shared = manifestShared;
	}

	return {
		...manifestOptions,
		remoteAliasMap,
		globalName,
		name: containerName
	};
}

/**
 * JS-side post-processing plugin: reads mf-manifest.json and mf-stats.json, executes additionalData callback and merges/overwrites manifest.
 * To avoid cross-NAPI callback complexity, this plugin runs at the afterProcessAssets stage to ensure Rust-side MfManifestPlugin has already output its artifacts.
 */
export class ModuleFederationManifestPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ModuleFederationManifestPlugin;
	private rawOpts: ModuleFederationPluginOptions;
	constructor(opts: ModuleFederationPluginOptions) {
		super();
		this.rawOpts = opts;
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const opts = normalizeManifestOptions(this.rawOpts);
		const {
			fileName,
			filePath,
			disableAssetsAnalyze,
			remoteAliasMap,
			exposes,
			shared
		} = opts;
		const { statsFileName, manifestFileName } = getFileName(opts);

		const rawOptions: RawModuleFederationManifestPluginOptions = {
			name: opts.name,
			globalName: opts.globalName,
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
				compiler,
				this.rawOpts
			)
		};
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
