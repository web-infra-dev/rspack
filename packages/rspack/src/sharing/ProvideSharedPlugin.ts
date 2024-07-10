import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawProvideOptions
} from "@rspack/binding";

import type { Compiler } from "../Compiler";
import {
	RspackBuiltinPlugin,
	createBuiltinPlugin
} from "../builtin-plugin/base";
import { parseOptions } from "../container/options";
import { ShareRuntimePlugin } from "./ShareRuntimePlugin";

export type ProvideSharedPluginOptions = {
	provides: Provides;
	shareScope?: string;
	enhanced?: boolean;
};
export type Provides = (ProvidesItem | ProvidesObject)[] | ProvidesObject;
export type ProvidesItem = string;
export type ProvidesObject = {
	[k: string]: ProvidesConfig | ProvidesItem;
};
export type ProvidesConfig = {
	eager?: boolean;
	shareKey: string;
	shareScope?: string;
	version?: false | string;
};

export class ProvideSharedPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ProvideSharedPlugin;
	_options;

	constructor(options: ProvideSharedPluginOptions) {
		super();
		this._options = {
			provides: parseOptions(
				options.provides,
				item => {
					if (Array.isArray(item))
						throw new Error("Unexpected array of provides");
					const result = {
						shareKey: item,
						version: undefined,
						shareScope: options.shareScope || "default",
						eager: false
					};
					return result;
				},
				item => ({
					shareKey: item.shareKey,
					version: item.version,
					shareScope: item.shareScope || options.shareScope || "default",
					eager: !!item.eager
				})
			),
			enhanced: options.enhanced ?? false
		};
	}

	raw(compiler: Compiler): BuiltinPlugin {
		new ShareRuntimePlugin(this._options.enhanced).apply(compiler);

		const rawOptions: RawProvideOptions[] = this._options.provides.map(
			([key, v]) => ({
				key,
				...v
			})
		);
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
