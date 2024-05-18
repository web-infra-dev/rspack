import { Compiler } from "../Compiler";
import {
	EntryRuntime,
	ExternalsType,
	externalsType,
	LibraryOptions
} from "../config";
import { Shared, SharePlugin } from "../sharing/SharePlugin";
import { ShareRuntimePlugin } from "../sharing/ShareRuntimePlugin";
import { isValidate } from "../util/validate";
import { ContainerPlugin, Exposes } from "./ContainerPlugin";
import { ContainerReferencePlugin, Remotes } from "./ContainerReferencePlugin";

export interface ModuleFederationPluginV1Options {
	exposes?: Exposes;
	filename?: string;
	library?: LibraryOptions;
	name: string;
	remoteType?: ExternalsType;
	remotes?: Remotes;
	runtime?: EntryRuntime;
	shareScope?: string;
	shared?: Shared;
	enhanced?: boolean;
}

export class ModuleFederationPluginV1 {
	constructor(private _options: ModuleFederationPluginV1Options) {}

	apply(compiler: Compiler) {
		const { _options: options } = this;
		const enhanced = options.enhanced ?? false;

		const library = options.library || { type: "var", name: options.name };
		const remoteType =
			options.remoteType ||
			(options.library && isValidate(options.library.type, externalsType)
				? (options.library.type as ExternalsType)
				: "script");
		if (
			library &&
			!compiler.options.output.enabledLibraryTypes!.includes(library.type)
		) {
			compiler.options.output.enabledLibraryTypes!.push(library.type);
		}
		compiler.hooks.afterPlugins.tap("ModuleFederationPlugin", () => {
			new ShareRuntimePlugin(this._options.enhanced).apply(compiler);
			if (
				options.exposes &&
				(Array.isArray(options.exposes)
					? options.exposes.length > 0
					: Object.keys(options.exposes).length > 0)
			) {
				new ContainerPlugin({
					name: options.name,
					library,
					filename: options.filename,
					runtime: options.runtime,
					shareScope: options.shareScope,
					exposes: options.exposes,
					enhanced
				}).apply(compiler);
			}
			if (
				options.remotes &&
				(Array.isArray(options.remotes)
					? options.remotes.length > 0
					: Object.keys(options.remotes).length > 0)
			) {
				new ContainerReferencePlugin({
					remoteType,
					shareScope: options.shareScope,
					remotes: options.remotes,
					enhanced
				}).apply(compiler);
			}
			if (options.shared) {
				new SharePlugin({
					shared: options.shared,
					shareScope: options.shareScope,
					enhanced
				}).apply(compiler);
			}
		});
	}
}
