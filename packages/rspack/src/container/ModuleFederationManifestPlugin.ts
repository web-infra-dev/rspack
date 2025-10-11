import { join } from "node:path";
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
	buildName: string;
}

interface MetaDataTypes {
	path: string;
	name: string;
	api: string;
	zip: string;
}

interface BasicStatsMetaData {
	name: string;
	globalName: string;
	buildInfo: StatsBuildInfo;
	remoteEntry: ResourceInfo;
	ssrRemoteEntry?: ResourceInfo;
	prefetchInterface?: boolean;
	prefetchEntry?: ResourceInfo;
	types: MetaDataTypes;
	type: string;
	pluginVersion: string;
}

type StatsMetaDataWithGetPublicPath = BasicStatsMetaData & {
	getPublicPath: string;
};

type StatsMetaDataWithPublicPath = BasicStatsMetaData & {
	publicPath: string;
	ssrPublicPath?: string;
};

type StatsMetaData =
	| StatsMetaDataWithGetPublicPath
	| StatsMetaDataWithPublicPath;

interface StatsAssets {
	js: StatsAssetsInfo;
	css: StatsAssetsInfo;
}

interface StatsAssetsInfo {
	sync: string[];
	async: string[];
}

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

export type ModuleFederationManifestPluginOptions = {
	name?: string;
	globalName?: string;
	filePath?: string;
	disableAssetsAnalyze?: boolean;
	fileName?: string;
	typesFileName?: string;
	getPublicPath?: string;
	additionalData?: (
		options: AdditionalDataOptions
	) => Promise<Stats | void> | Stats | void;
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

	const JSON_EXT = ".json";
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
			compilation.hooks.afterProcessAssets.tap(this.name, () => {
				const manifestAsset = compilation.getAsset(manifestFileName);
				const statsAsset = compilation.getAsset(statsFileName);
				if (!manifestAsset || !statsAsset) {
					return;
				}
				const manifestStr = toStringSource(manifestAsset.source);
				const statsStr = toStringSource(statsAsset.source);
				if (!manifestStr || !statsStr) return;
				let manifestObj: Manifest;
				let statsObj: Stats;
				try {
					manifestObj = JSON.parse(manifestStr);
					statsObj = JSON.parse(statsStr);
				} catch (_e) {
					return;
				}
				if (typeof this.opts.additionalData !== "function") return;
				let patch: any;
				try {
					patch = this.opts.additionalData({
						stats: statsObj,
						manifest: manifestObj,
						compilation,
						compiler,
						bundler: "rspack"
					});
				} catch (_e) {
					return;
				}
				if (!patch || typeof patch !== "object") return;

				const nextManifest = patch || manifestObj;
				const nextSource = new RawSource(JSON.stringify(nextManifest, null, 2));
				compilation.updateAsset(manifestFileName, nextSource);
			});
		});
	}

	raw(): BuiltinPlugin {
		const { fileName, filePath, disableAssetsAnalyze } = this.opts;
		const { statsFileName } = getFileName(this.opts);

		const rawOptions: RawModuleFederationManifestPluginOptions = {
			name: this.opts.name,
			globalName: this.opts.globalName,
			fileName,
			filePath,
			statsFileName,
			disableAssetsAnalyze,
			typesFileName: this.opts.typesFileName,
			getPublicPath: this.opts.getPublicPath
		};
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
