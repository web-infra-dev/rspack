import {
	BuiltinPlugin,
	RawContainerReferencePluginOptions
} from "@rspack/binding";
import { BuiltinPluginName, RspackBuiltinPlugin } from "../builtin-plugin/base";
import { Compiler } from "../Compiler";
import { ExternalsPlugin } from "../builtin-plugin/ExternalsPlugin";
import { ExternalsType } from "../config";
import { parseOptions } from "./options";
import { ModuleFederationRuntimePlugin } from "./ModuleFederationRuntimePlugin";
import { isNil } from "../util";

export type ContainerReferencePluginOptions = {
	remoteType: ExternalsType;
	remotes: Remotes;
	shareScope?: string;
};
export type Remotes = (RemotesItem | RemotesObject)[] | RemotesObject;
export type RemotesItem = string;
export type RemotesItems = RemotesItem[];
export type RemotesObject = {
	[k: string]: RemotesConfig | RemotesItem | RemotesItems;
};
export type RemotesConfig = {
	external: RemotesItem | RemotesItems;
	shareScope?: string;
};

export class ContainerReferencePlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ContainerReferencePlugin;
	_options: RawContainerReferencePluginOptions;
	_remotes;
	_mfRuntimePlugin: [boolean, ModuleFederationRuntimePlugin];

	constructor(
		options: ContainerReferencePluginOptions,
		mfRuntimePlugin?: ModuleFederationRuntimePlugin
	) {
		super();
		this._remotes = parseOptions(
			options.remotes,
			item => ({
				external: Array.isArray(item) ? item : [item],
				shareScope: options.shareScope || "default"
			}),
			item => ({
				external: Array.isArray(item.external)
					? item.external
					: [item.external],
				shareScope: item.shareScope || options.shareScope || "default"
			})
		);
		this._options = {
			remoteType: options.remoteType,
			remotes: this._remotes.map(([key, r]) => ({ key, ...r }))
		};
		this._mfRuntimePlugin = [
			!isNil(mfRuntimePlugin),
			mfRuntimePlugin ?? new ModuleFederationRuntimePlugin()
		];
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const { remoteType } = this._options;
		const remoteExternals: any = {};
		for (const [key, config] of this._remotes) {
			let i = 0;
			for (const external of config.external) {
				if (external.startsWith("internal ")) continue;
				remoteExternals[
					`webpack/container/reference/${key}${i ? `/fallback-${i}` : ""}`
				] = external;
				i++;
			}
		}
		new ExternalsPlugin(remoteType, remoteExternals).apply(compiler);
		const [injected, mfRuntimePlugin] = this._mfRuntimePlugin;
		mfRuntimePlugin.addPlugin(
			require.resolve("../sharing/initializeSharing.js")
		);
		mfRuntimePlugin.addPlugin(require.resolve("./remotesLoading.js"));
		if (!injected) {
			mfRuntimePlugin.apply(compiler);
		}

		return {
			name: this.name as any,
			options: this._options
		};
	}
}
