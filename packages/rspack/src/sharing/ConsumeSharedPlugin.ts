import {
	BuiltinPlugin,
	BuiltinPluginName,
	RawConsumeSharedPluginOptions
} from "@rspack/binding";
import { Compiler } from "../Compiler";
import {
	RspackBuiltinPlugin,
	createBuiltinPlugin
} from "../builtin-plugin/base";
import { parseOptions } from "../container/options";
import { isRequiredVersion } from "./utils";
import { ShareRuntimePlugin } from "./ShareRuntimePlugin";

export type ConsumeSharedPluginOptions = {
	consumes: Consumes;
	shareScope?: string;
	enhanced?: boolean;
};
export type Consumes = (ConsumesItem | ConsumesObject)[] | ConsumesObject;
export type ConsumesItem = string;
export type ConsumesObject = {
	[k: string]: ConsumesConfig | ConsumesItem;
};
export type ConsumesConfig = {
	eager?: boolean;
	import?: false | ConsumesItem;
	packageName?: string;
	requiredVersion?: false | string;
	shareKey?: string;
	shareScope?: string;
	singleton?: boolean;
	strictVersion?: boolean;
};

export class ConsumeSharedPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ConsumeSharedPlugin;
	_options;

	constructor(options: ConsumeSharedPluginOptions) {
		super();
		this._options = {
			consumes: parseOptions(
				options.consumes,
				(item, key) => {
					if (Array.isArray(item))
						throw new Error("Unexpected array in options");
					let result =
						item === key || !isRequiredVersion(item)
							? // item is a request/key
								{
									import: key,
									shareScope: options.shareScope || "default",
									shareKey: key,
									requiredVersion: undefined,
									packageName: undefined,
									strictVersion: false,
									singleton: false,
									eager: false
								}
							: // key is a request/key
								// item is a version
								{
									import: key,
									shareScope: options.shareScope || "default",
									shareKey: key,
									requiredVersion: item,
									strictVersion: true,
									packageName: undefined,
									singleton: false,
									eager: false
								};
					return result;
				},
				(item, key) => ({
					import: item.import === false ? undefined : item.import || key,
					shareScope: item.shareScope || options.shareScope || "default",
					shareKey: item.shareKey || key,
					requiredVersion: item.requiredVersion,
					strictVersion:
						typeof item.strictVersion === "boolean"
							? item.strictVersion
							: item.import !== false && !item.singleton,
					packageName: item.packageName,
					singleton: !!item.singleton,
					eager: !!item.eager
				})
			),
			enhanced: options.enhanced ?? false
		};
	}

	raw(compiler: Compiler): BuiltinPlugin {
		new ShareRuntimePlugin(this._options.enhanced).apply(compiler);

		const rawOptions: RawConsumeSharedPluginOptions = {
			consumes: this._options.consumes.map(([key, v]) => ({
				key,
				...v
			})),
			enhanced: this._options.enhanced
		};
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
