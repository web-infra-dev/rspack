import { ExternalsType } from "../config";
import { Compiler } from "../Compiler";
import { type ModuleFederationPluginV1Options } from "./ModuleFederationPluginV1";

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

function getRemoteInfos(options: ModuleFederationPluginOptions) {
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
			return [external.slice(0, idx), external.slice(idx + 1)];
		}
		return null;
	}

	function getExternal(external: string, defaultExternalType: ExternalsType) {
		const result = getExternalTypeFromExternal(external);
		if (result === null) {
			return [defaultExternalType, external];
		}
		return result;
	}

	const remotes: Record<string, unknown> = {};
	const remoteType =
		options.remoteType ||
		(options.library ? (options.library.type as ExternalsType) : "script");
	for (let [key, remote] of Object.entries(options.remotes ?? {})) {
		const [externalType, external] = getExternal(
			typeof remote === "string" ? remote : remote.external,
			remoteType
		);
		const shareScope =
			(typeof remote !== "string" && remote.shareScope) ||
			options.shareScope ||
			"default";
		if (externalType === "script") {
			const [url, global] = extractUrlAndGlobal(external)!;
			remotes[key] = {
				alias: key,
				name: global,
				entry: url,
				externalType,
				shareScope
			};
		} else {
			remotes[key] = {
				alias: key,
				name: undefined,
				entry: undefined,
				externalType,
				shareScope
			};
		}
	}
	return remotes;
}

function getRuntimePlugins(options: ModuleFederationPluginOptions) {
	return options.runtimePlugins ?? [];
}

function getImplementation(options: ModuleFederationPluginOptions) {
	return (
		options.implementation ??
		require.resolve("@module-federation/webpack-bundler-runtime")
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
