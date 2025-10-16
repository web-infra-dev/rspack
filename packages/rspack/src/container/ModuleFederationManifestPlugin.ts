import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawModuleFederationManifestPluginOptions
} from "@rspack/binding";
import type { Source } from "webpack-sources";
import { RawSource } from "webpack-sources";
import {
	createBuiltinPlugin,
	RspackBuiltinPlugin
} from "../builtin-plugin/base";
import type { Compilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import type { LibraryType } from "../config";

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
		buildVersion: process.env.MF_BUILD_VERSION || buildVersion,
		buildName: process.env.MF_BUILD_NAME || pkg?.name || ""
	};
}

interface RemoteWithEntry {
	name: string;
	entry: string;
}

interface RemoteWithVersion {
	name: string;
	version: string;
}

interface ResourceInfo {
	path: string;
	name: string;
	type: LibraryType;
}

interface StatsBuildInfo {
	buildVersion: string;
	buildName?: string;
}

interface BasicStatsMetaData {
	name: string;
	globalName: string;
	buildInfo: StatsBuildInfo;
	remoteEntry: ResourceInfo;
	type: string;
}

type StatsMetaDataWithPublicPath = BasicStatsMetaData & {
	publicPath: string;
};

type StatsMetaData = StatsMetaDataWithPublicPath;

interface StatsAssets {
	js: StatsAssetsInfo;
	css: StatsAssetsInfo;
}

interface StatsAssetsInfo {
	sync: string[];
	async: string[];
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

interface StatsShared {
	id: string;
	name: string;
	version: string;
	singleton: boolean;
	requiredVersion: string;
	hash: string;
	assets: StatsAssets;
	deps: string[];
	usedIn: string[];
}
interface StatsRemoteVal {
	moduleName: string;
	federationContainerName: string;
	consumingFederationContainerName: string;
	alias: string;
	usedIn: string[];
}

type StatsRemoteWithEntry = StatsRemoteVal & Omit<RemoteWithEntry, "name">;
type StatsRemoteWithVersion = StatsRemoteVal & Omit<RemoteWithVersion, "name">;

type StatsRemote = StatsRemoteWithEntry | StatsRemoteWithVersion;

interface StatsExpose {
	id: string;
	name: string;
	path?: string;
	file: string;
	requires: string[];
	assets: StatsAssets;
}

interface Stats {
	id: string;
	name: string;
	metaData: StatsMetaData;
	shared: StatsShared[];
	remotes: StatsRemote[];
	exposes: StatsExpose[];
}

interface ManifestShared {
	id: string;
	name: string;
	version: string;
	singleton: boolean;
	requiredVersion: string;
	hash: string;
	assets: StatsAssets;
}

interface ManifestRemoteCommonInfo {
	federationContainerName: string;
	moduleName: string;
	alias: string;
}

type ManifestRemote =
	| (Omit<RemoteWithEntry, "name"> & ManifestRemoteCommonInfo)
	| (Omit<RemoteWithVersion, "name"> & ManifestRemoteCommonInfo);

type ManifestExpose = Pick<StatsExpose, "assets" | "id" | "name" | "path">;

interface Manifest {
	id: string;
	name: string;
	metaData: StatsMetaData;
	shared: ManifestShared[];
	remotes: ManifestRemote[];
	exposes: ManifestExpose[];
}

interface AdditionalDataOptions {
	stats: Stats;
	manifest?: Manifest;
	compiler: Compiler;
	compilation: Compilation;
	bundler: "webpack" | "rspack";
}

function isStats(value: unknown): value is Stats {
	if (!isPlainObject(value)) return false;
	if (typeof value.id !== "string" || typeof value.name !== "string") {
		return false;
	}
	if (!isPlainObject(value.metaData)) return false;
	if (!Array.isArray(value.shared)) return false;
	if (!Array.isArray(value.remotes)) return false;
	if (!Array.isArray(value.exposes)) return false;
	return true;
}

function isManifest(value: unknown): value is Manifest {
	if (!isPlainObject(value)) return false;
	if (typeof value.id !== "string" || typeof value.name !== "string") {
		return false;
	}
	if (!isPlainObject(value.metaData)) return false;
	if (!Array.isArray(value.shared)) return false;
	if (!Array.isArray(value.remotes)) return false;
	if (!Array.isArray(value.exposes)) return false;
	return true;
}

export type ModuleFederationManifestPluginOptions = {
	name?: string;
	globalName?: string;
	filePath?: string;
	disableAssetsAnalyze?: boolean;
	fileName?: string;
	remoteAliasMap?: RemoteAliasMap;
	exposes?: ManifestExposeOption[];
	shared?: ManifestSharedOption[];
	additionalData?: (options: AdditionalDataOptions) => Promise<void> | void;
};

function toStringSource(source: Source | void): string | undefined {
	if (!source) return undefined;
	const content = source.source();
	if (typeof content === "string") return content;
	if (Buffer.isBuffer(content)) return content.toString("utf-8");
	return String(content);
}

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

	apply(compiler: Compiler) {
		super.apply(compiler);
		const { statsFileName, manifestFileName } = getFileName(this.opts);

		compiler.hooks.thisCompilation.tap(this.name, compilation => {
			compilation.hooks.processAssets.tapPromise(this.name, async () => {
				if (typeof this.opts.additionalData !== "function") return;

				const manifestAsset = compilation.getAsset(manifestFileName);
				const statsAsset = compilation.getAsset(statsFileName);
				if (!manifestAsset || !statsAsset) {
					return;
				}
				const manifestStr = toStringSource(manifestAsset.source);
				const statsStr = toStringSource(statsAsset.source);
				if (!manifestStr || !statsStr) return;

				const manifest = parseJSON(manifestStr, isManifest);
				const stats = parseJSON(statsStr, isStats);
				if (!manifest || !stats) return;

				await this.opts.additionalData({
					stats,
					manifest,
					compilation,
					compiler,
					bundler: "rspack"
				});

				compilation.updateAsset(
					manifestFileName,
					new RawSource(JSON.stringify(manifest, null, 2))
				);
				compilation.updateAsset(
					statsFileName,
					new RawSource(JSON.stringify(stats, null, 2))
				);
			});
		});
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
