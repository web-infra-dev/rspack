import type { Compiler } from "../Compiler";
import type { ExternalsType } from "../config";
import type { SharedConfig } from "../sharing/SharePlugin";
import { isRequiredVersion } from "../sharing/utils";
import {
	type ManifestExposeOption,
	type ManifestSharedOption,
	ModuleFederationManifestPlugin,
	type ModuleFederationManifestPluginOptions,
	type RemoteAliasMap
} from "./ModuleFederationManifestPlugin";
import type { ModuleFederationPluginV1Options } from "./ModuleFederationPluginV1";
import {
	type ModuleFederationRuntimeExperimentsOptions,
	ModuleFederationRuntimePlugin
} from "./ModuleFederationRuntimePlugin";
import { parseOptions } from "./options";

declare const MF_RUNTIME_CODE: string;

export interface ModuleFederationPluginOptions
	extends Omit<ModuleFederationPluginV1Options, "enhanced"> {
	runtimePlugins?: RuntimePlugins;
	implementation?: string;
	shareStrategy?: "version-first" | "loaded-first";
	manifest?:
		| boolean
		| Omit<
				ModuleFederationManifestPluginOptions,
				"remoteAliasMap" | "globalName" | "name" | "exposes" | "shared"
		  >;
	experiments?: ModuleFederationRuntimeExperimentsOptions;
}
export type RuntimePlugins = string[] | [string, Record<string, unknown>][];

export class ModuleFederationPlugin {
	constructor(private _options: ModuleFederationPluginOptions) {}

	apply(compiler: Compiler) {
		const { webpack } = compiler;
		const paths = getPaths(this._options);
		compiler.options.resolve.alias = {
			"@module-federation/runtime-tools": paths.runtimeTools,
			"@module-federation/runtime": paths.runtime,
			...compiler.options.resolve.alias
		};

		// Generate the runtime entry content
		const entryRuntime = getDefaultEntryRuntime(paths, this._options, compiler);

		// Pass only the entry runtime to the Rust-side plugin
		new ModuleFederationRuntimePlugin({
			entryRuntime,
			experiments: this._options.experiments
		}).apply(compiler);

		new webpack.container.ModuleFederationPluginV1({
			...this._options,
			enhanced: true
		}).apply(compiler);

		if (this._options.manifest) {
			const manifestOptions: ModuleFederationManifestPluginOptions =
				this._options.manifest === true ? {} : { ...this._options.manifest };
			const containerName = manifestOptions.name ?? this._options.name;
			const globalName =
				manifestOptions.globalName ??
				resolveLibraryGlobalName(this._options.library) ??
				containerName;
			const remoteAliasMap: RemoteAliasMap = Object.entries(
				getRemoteInfos(this._options)
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

			const manifestExposes = collectManifestExposes(this._options.exposes);
			if (manifestOptions.exposes === undefined && manifestExposes) {
				manifestOptions.exposes = manifestExposes;
			}
			const manifestShared = collectManifestShared(this._options.shared);
			if (manifestOptions.shared === undefined && manifestShared) {
				manifestOptions.shared = manifestShared;
			}

			new ModuleFederationManifestPlugin({
				...manifestOptions,
				name: containerName,
				globalName,
				remoteAliasMap
			}).apply(compiler);
		}
	}
}

interface RuntimePaths {
	runtimeTools: string;
	bundlerRuntime: string;
	runtime: string;
}

interface RemoteInfo {
	alias: string;
	name?: string;
	entry?: string;
	externalType: ExternalsType;
	shareScope: string;
}

type RemoteInfos = Record<string, RemoteInfo[]>;

function collectManifestExposes(
	exposes: ModuleFederationPluginOptions["exposes"]
): ManifestExposeOption[] | undefined {
	if (!exposes) return undefined;
	type NormalizedExpose = { import: string[]; name?: string };
	type ExposesConfigInput = { import: string | string[]; name?: string };
	const parsed = parseOptions<ExposesConfigInput, NormalizedExpose>(
		exposes,
		(value, key) => ({
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

function getRemoteInfos(options: ModuleFederationPluginOptions): RemoteInfos {
	if (!options.remotes) {
		return {};
	}

	function extractUrlAndGlobal(urlAndGlobal: string) {
		const index = urlAndGlobal.indexOf("@");
		if (index <= 0 || index === urlAndGlobal.length - 1) {
			return null;
		}
		return [
			urlAndGlobal.substring(index + 1),
			urlAndGlobal.substring(0, index)
		] as const;
	}

	function getExternalTypeFromExternal(external: string) {
		if (/^[a-z0-9-]+ /.test(external)) {
			const idx = external.indexOf(" ");
			return [
				external.slice(0, idx) as ExternalsType,
				external.slice(idx + 1)
			] as const;
		}
		return null;
	}

	function getExternal(external: string) {
		const result = getExternalTypeFromExternal(external);
		if (result === null) {
			return [remoteType, external] as const;
		}
		return result;
	}

	const remoteType =
		options.remoteType ||
		(options.library ? (options.library.type as ExternalsType) : "script");

	const remotes = parseOptions(
		options.remotes,
		item => ({
			external: Array.isArray(item) ? item : [item],
			shareScope: options.shareScope || "default"
		}),
		item => ({
			external: Array.isArray(item.external) ? item.external : [item.external],
			shareScope: item.shareScope || options.shareScope || "default"
		})
	);

	const remoteInfos: Record<string, RemoteInfo[]> = {};
	for (const [key, config] of remotes) {
		for (const external of config.external) {
			const [externalType, externalRequest] = getExternal(external);
			remoteInfos[key] ??= [];
			if (externalType === "script") {
				const [url, global] = extractUrlAndGlobal(externalRequest)!;
				remoteInfos[key].push({
					alias: key,
					name: global,
					entry: url,
					externalType,
					shareScope: config.shareScope
				});
			} else {
				remoteInfos[key].push({
					alias: key,
					name: undefined,
					entry: undefined,
					externalType,
					shareScope: config.shareScope
				});
			}
		}
	}
	return remoteInfos;
}

function getRuntimePlugins(options: ModuleFederationPluginOptions) {
	return options.runtimePlugins ?? [];
}

function getPaths(options: ModuleFederationPluginOptions): RuntimePaths {
	if (IS_BROWSER) {
		return {
			runtimeTools: "@module-federation/runtime-tools",
			bundlerRuntime: "@module-federation/webpack-bundler-runtime",
			runtime: "@module-federation/runtime"
		};
	}

	const runtimeToolsPath =
		options.implementation ??
		require.resolve("@module-federation/runtime-tools");
	const bundlerRuntimePath = require.resolve(
		"@module-federation/webpack-bundler-runtime",
		{ paths: [runtimeToolsPath] }
	);
	const runtimePath = require.resolve("@module-federation/runtime", {
		paths: [runtimeToolsPath]
	});
	return {
		runtimeTools: runtimeToolsPath,
		bundlerRuntime: bundlerRuntimePath,
		runtime: runtimePath
	};
}

function getDefaultEntryRuntime(
	paths: RuntimePaths,
	options: ModuleFederationPluginOptions,
	compiler: Compiler
) {
	const runtimePlugins = getRuntimePlugins(options);
	const remoteInfos = getRemoteInfos(options);
	const runtimePluginImports = [];
	const runtimePluginVars = [];
	for (let i = 0; i < runtimePlugins.length; i++) {
		const runtimePluginVar = `__module_federation_runtime_plugin_${i}__`;
		const pluginSpec = runtimePlugins[i];
		const pluginPath = Array.isArray(pluginSpec) ? pluginSpec[0] : pluginSpec;
		const pluginParams = Array.isArray(pluginSpec) ? pluginSpec[1] : undefined;

		runtimePluginImports.push(
			`import ${runtimePluginVar} from ${JSON.stringify(pluginPath)}`
		);
		const paramsCode =
			pluginParams === undefined ? "undefined" : JSON.stringify(pluginParams);
		runtimePluginVars.push(`${runtimePluginVar}(${paramsCode})`);
	}

	const content = [
		`import __module_federation_bundler_runtime__ from ${JSON.stringify(
			paths.bundlerRuntime
		)}`,
		...runtimePluginImports,
		`const __module_federation_runtime_plugins__ = [${runtimePluginVars.join(
			", "
		)}]`,
		`const __module_federation_remote_infos__ = ${JSON.stringify(remoteInfos)}`,
		`const __module_federation_container_name__ = ${JSON.stringify(
			options.name ?? compiler.options.output.uniqueName
		)}`,
		`const __module_federation_share_strategy__ = ${JSON.stringify(
			options.shareStrategy ?? "version-first"
		)}`,
		IS_BROWSER
			? MF_RUNTIME_CODE
			: compiler.webpack.Template.getFunctionContent(
					require("./moduleFederationDefaultRuntime.js")
				)
	].join(";");
	return `@module-federation/runtime/rspack.js!=!data:text/javascript,${content}`;
}
