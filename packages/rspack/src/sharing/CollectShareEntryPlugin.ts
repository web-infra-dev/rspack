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

	// private async collectBeforeRun(compiler: Compiler) {
	// 	if (!this._provides.length) {
	// 		this._collectedEntries = {};
	// 		return;
	// 	}

	// 	const resolver = compiler.resolverFactory.get("normal", {
	// 		dependencyType: "esm"
	// 	});
	// 	const context =
	// 		compiler.options.context || compiler.context || process.cwd();

	// 	const collected: typeof this._collectedEntries = {};
	// 	const normalize = (resolved: string) => resolved.replace(/\u200b/g, "");

	// 	await Promise.all(
	// 		this._provides.map(async ([request, info]) => {
	// 			try {
	// 				if (request.endsWith("/")) {
	// 					return;
	// 				}
	// 				const resolved = await new Promise<string | null>(resolve => {
	// 					resolver.resolve(
	// 						{},
	// 						context,
	// 						request,
	// 						{},
	// 						(err, result) => {
	// 							if (err || !result) {
	// 								return resolve(null);
	// 							}
	// 							resolve(result);
	// 						}
	// 					);
	// 				});
	// 				if (!resolved) {
	// 					return;
	// 				}
	// 				const normalized = normalize(resolved);
	// 				const relative = path.relative(context, normalized);
	// 				const normalizedResource = relative.startsWith("..")
	// 					? normalized
	// 					: relative || normalized;
	// 				const posix = normalizedResource.split(path.sep).join("/");
	// 				const requests = collected[info.shareKey]?.requests ?? [];
	// 				if (!requests.some(([req]) => req === posix)) {
	// 					requests.push([posix, "unknown"]);
	// 				}
	// 				collected[info.shareKey] = {
	// 					shareScope: info.shareScope,
	// 					requests
	// 				};
	// 			} catch {
	// 				// ignore resolution failures at this stage
	// 			}
	// 		})
	// 	);

	// 	this._collectedEntries = collected;
	// }

	apply(compiler: Compiler) {
		super.apply(compiler);

		// compiler.hooks.beforeRun.tapPromise(
		// 	{ name: "CollectShareEntryPlugin", stage: -100 },
		// 	async () => {
		// 		await this.collectBeforeRun(compiler);
		// 	}
		// );
		compiler.hooks.thisCompilation.tap("Collect share entry", compilation => {
			compilation.hooks.processAssets.tapPromise(
				{
					name: "collect share entry",
					stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
				},
				async () => {
					const filename = this.getFilename();
					const asset = compilation.getAsset(filename);
					if (!asset) {
						throw new Error(`Can not get ${filename}`);
					}
					this._collectedEntries = JSON.parse(asset.source.source().toString());
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
