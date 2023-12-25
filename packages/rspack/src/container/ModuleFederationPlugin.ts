import { ExternalsType, externalsType } from "../config";
import { Compiler } from "../Compiler";
import { type ModuleFederationPluginV1Options } from "./ModuleFederationPluginV1";
import { parseOptions } from "./options";
import { isValidate } from "../util/validate";

export interface ModuleFederationPluginOptions
	extends Omit<ModuleFederationPluginV1Options, "enhanced"> {
	runtimePlugins?: string[];
	implementation?: string;
}

export class ModuleFederationPlugin {
	constructor(private _options: ModuleFederationPluginOptions) {}

	apply(compiler: Compiler) {
		const { webpack } = compiler;
		new webpack.EntryPlugin(
			compiler.context,
			getDefaultEntryRuntime(this._options, compiler),
			{ name: undefined }
		).apply(compiler);
		new webpack.container.ModuleFederationPluginV1({
			...this._options,
			enhanced: true
		}).apply(compiler);
	}
}

interface RemoteInfo {
	alias: string;
	name?: string;
	entry?: string;
	externalType: ExternalsType;
	shareScope: string;
}

type RemoteInfos = Record<string, RemoteInfo[]>;

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
		(options.library && isValidate(options.library.type, externalsType)
			? (options.library.type as ExternalsType)
			: "script");
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
	for (let [key, config] of remotes) {
		for (let external of config.external) {
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

function getImplementation(options: ModuleFederationPluginOptions) {
	return (
		options.implementation ??
		require.resolve("@module-federation/runtime-tools/webpack-bundler-runtime")
	);
}

function getDefaultEntryRuntime(
	options: ModuleFederationPluginOptions,
	compiler: Compiler
) {
	const implementationPath = getImplementation(options);
	const runtimePlugins = getRuntimePlugins(options);
	const remoteInfos = getRemoteInfos(options);
	const runtimePluginImports = [];
	const runtimePluginVars = [];
	for (let i = 0; i < runtimePlugins.length; i++) {
		const runtimePluginVar = `__module_federation_runtime_plugin_${i}__`;
		runtimePluginImports.push(
			`import ${runtimePluginVar} from ${JSON.stringify(runtimePlugins[i])}`
		);
		runtimePluginVars.push(`${runtimePluginVar}()`);
	}
	const content = [
		`import __module_federation_runtime__ from ${JSON.stringify(
			implementationPath
		)}`,
		...runtimePluginImports,
		`const __module_federation_runtime_plugins__ = [${runtimePluginVars.join(
			", "
		)}]`,
		`const __module_federation_remote_infos__ = ${JSON.stringify(remoteInfos)}`,
		compiler.webpack.Template.getFunctionContent(require("./default.runtime"))
	].join("\n");
	return `data:text/javascript,${content}`;
}
