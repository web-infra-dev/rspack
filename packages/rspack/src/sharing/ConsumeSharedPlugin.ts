import { BuiltinPlugin, RawConsumeOptions } from "@rspack/binding";
import { Compiler } from "../Compiler";
import { BuiltinPluginName, RspackBuiltinPlugin } from "../builtin-plugin/base";
import { parseOptions } from "../container/options";
import { isRequiredVersion } from "./utils";

export type ConsumeSharedPluginOptions = {
	consumes: Consumes;
	shareScope?: string;
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
	_consumes;

	constructor(options: ConsumeSharedPluginOptions) {
		super();
		this._consumes = parseOptions(
			options.consumes,
			(item, key) => {
				if (Array.isArray(item)) throw new Error("Unexpected array in options");
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
		);
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const rawOptions: RawConsumeOptions[] = this._consumes.map(([key, v]) => ({
			key,
			...v
		}));
		return {
			name: this.name as any,
			options: rawOptions
		};
	}
}
