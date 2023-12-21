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
			defaultImplementation(this._options, compiler),
			{ name: undefined }
		).apply(compiler);
		new webpack.container.ModuleFederationPluginV1({
			...this._options,
			enhanced: true
		}).apply(compiler);
	}
}

function defaultImplementation(
	options: ModuleFederationPluginOptions,
	compiler: Compiler
) {
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

	const runtimeTemplate = compiler.webpack.Template.getFunctionContent(
		require("./default.runtime.js")
	);
	const remotes = [];
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
			remotes.push({
				alias: key,
				name: global,
				entry: url,
				externalType,
				shareScope
			});
		} else {
			remotes.push({
				alias: key,
				name: undefined,
				entry: undefined,
				externalType,
				shareScope
			});
		}
	}
	const runtimePlugins = options.runtimePlugins ?? [];
	const pluginImports = runtimePlugins.map(
		p => `require(${JSON.stringify(p)})`
	);
	const implementationPath =
		options.implementation ??
		require.resolve("@module-federation/webpack-bundler-runtime");
	let implementation = runtimeTemplate
		.replace("$RUNTIME_PACKAGE_PATH$", JSON.stringify(implementationPath))
		.replace("$ALL_REMOTES$", JSON.stringify(remotes))
		.replace("$INITOPTIONS_PLUGINS$", `[${pluginImports.join(", ")}]`);
	return `data:text/javascript,${implementation}`;
}
