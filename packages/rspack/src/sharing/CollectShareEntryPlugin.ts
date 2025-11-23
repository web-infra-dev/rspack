import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawCollectShareEntryPluginOptions
} from "@rspack/binding";
import {
	createBuiltinPlugin,
	RspackBuiltinPlugin
} from "../builtin-plugin/base";
import type { Compiler } from "../Compiler";
import { normalizeConsumeShareOptions } from "./ConsumeSharedPlugin";
import {
	createConsumeShareOptions,
	type NormalizedSharedOptions
} from "./SharePlugin";

export type CollectShareEntryPluginOptions = {
	sharedOptions: NormalizedSharedOptions;
	shareScope?: string;
};

export type ShareRequestsMap = Record<
	string,
	{
		shareScope: string;
		requests: [string, string][];
	}
>;

const SHARE_ENTRY_ASSET = "collect-share-entries.json";
export class CollectShareEntryPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.CollectShareEntryPlugin;
	sharedOptions: NormalizedSharedOptions;
	private _collectedEntries: ShareRequestsMap;

	constructor(options: CollectShareEntryPluginOptions) {
		super();
		const { sharedOptions } = options;

		this.sharedOptions = sharedOptions;
		this._collectedEntries = {};
	}

	getData() {
		return this._collectedEntries;
	}

	getFilename() {
		return SHARE_ENTRY_ASSET;
	}

	apply(compiler: Compiler) {
		super.apply(compiler);

		compiler.hooks.thisCompilation.tap("Collect share entry", compilation => {
			compilation.hooks.processAssets.tapPromise(
				{
					name: "CollectShareEntry",
					stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
				},
				async () => {
					const filename = this.getFilename();
					const asset = compilation.getAsset(filename);
					if (!asset) {
						throw new Error(`Can not get ${filename}`);
					}
					const parsed = JSON.parse(asset.source.source().toString());
					this._collectedEntries = parsed.shared ?? parsed;
				}
			);
		});
	}

	raw(): BuiltinPlugin {
		const consumeShareOptions = createConsumeShareOptions(this.sharedOptions);
		const normalizedConsumeShareOptions =
			normalizeConsumeShareOptions(consumeShareOptions);
		const rawOptions: RawCollectShareEntryPluginOptions = {
			consumes: normalizedConsumeShareOptions.map(([key, v]) => ({
				key,
				...v
			})),
			filename: this.getFilename()
		};
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
