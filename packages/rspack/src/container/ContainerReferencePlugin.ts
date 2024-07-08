import {
	BuiltinPlugin,
	BuiltinPluginName,
	RawContainerReferencePluginOptions
} from "@rspack/binding";

import { Compiler } from "../Compiler";
import { ExternalsPlugin } from "../builtin-plugin/ExternalsPlugin";
import {
	RspackBuiltinPlugin,
	createBuiltinPlugin
} from "../builtin-plugin/base";
import { ExternalsType } from "../config";
import { ShareRuntimePlugin } from "../sharing/ShareRuntimePlugin";
import { parseOptions } from "./options";

export type ContainerReferencePluginOptions = {
	remoteType: ExternalsType;
	remotes: Remotes;
	shareScope?: string;
	enhanced?: boolean;
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
	_options;

	constructor(options: ContainerReferencePluginOptions) {
		super();
		this._options = {
			remoteType: options.remoteType,
			remotes: parseOptions(
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
			),
			enhanced: options.enhanced ?? false
		};
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const { remoteType, remotes } = this._options;
		const remoteExternals: any = {};
		for (const [key, config] of remotes) {
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
		new ShareRuntimePlugin(this._options.enhanced).apply(compiler);

		const rawOptions: RawContainerReferencePluginOptions = {
			remoteType: this._options.remoteType,
			remotes: this._options.remotes.map(([key, r]) => ({ key, ...r })),
			enhanced: this._options.enhanced
		};
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
