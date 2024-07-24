import type { Compiler } from "../Compiler";
import { parseOptions } from "../container/options";
import { ConsumeSharedPlugin } from "./ConsumeSharedPlugin";
import { ProvideSharedPlugin } from "./ProvideSharedPlugin";
import { isRequiredVersion } from "./utils";

export type SharePluginOptions = {
	shareScope?: string;
	shared: Shared;
	enhanced: boolean;
};
export type Shared = (SharedItem | SharedObject)[] | SharedObject;
export type SharedItem = string;
export type SharedObject = {
	[k: string]: SharedConfig | SharedItem;
};
export type SharedConfig = {
	eager?: boolean;
	import?: false | SharedItem;
	packageName?: string;
	requiredVersion?: false | string;
	shareKey?: string;
	shareScope?: string;
	singleton?: boolean;
	strictVersion?: boolean;
	version?: false | string;
};

export class SharePlugin {
	_shareScope;
	_consumes;
	_provides;
	_enhanced;

	constructor(options: SharePluginOptions) {
		const sharedOptions = parseOptions(
			options.shared,
			(item, key) => {
				if (typeof item !== "string")
					throw new Error("Unexpected array in shared");
				const config: SharedConfig =
					item === key || !isRequiredVersion(item)
						? {
								import: item
							}
						: {
								import: key,
								requiredVersion: item
							};
				return config;
			},
			item => item
		);
		const consumes = sharedOptions.map(([key, options]) => ({
			[key]: {
				import: options.import,
				shareKey: options.shareKey || key,
				shareScope: options.shareScope,
				requiredVersion: options.requiredVersion,
				strictVersion: options.strictVersion,
				singleton: options.singleton,
				packageName: options.packageName,
				eager: options.eager
			}
		}));
		const provides = sharedOptions
			.filter(([, options]) => options.import !== false)
			.map(([key, options]) => ({
				[options.import || key]: {
					shareKey: options.shareKey || key,
					shareScope: options.shareScope,
					version: options.version,
					eager: options.eager
				}
			}));
		this._shareScope = options.shareScope;
		this._consumes = consumes;
		this._provides = provides;
		this._enhanced = options.enhanced ?? false;
	}

	apply(compiler: Compiler) {
		new ConsumeSharedPlugin({
			shareScope: this._shareScope,
			consumes: this._consumes,
			enhanced: this._enhanced
		}).apply(compiler);
		new ProvideSharedPlugin({
			shareScope: this._shareScope,
			provides: this._provides,
			enhanced: this._enhanced
		}).apply(compiler);
	}
}
