export * from "./build";
import * as binding from "@rspack/binding";
import type { ExternalObject, RspackInternal } from "@rspack/binding";
import * as tapable from "tapable";
import * as config from "./config";
import type { RspackOptions, ResolvedRspackOptions, Assets } from "./config";
interface RspackThreadsafeContext<T> {
	readonly id: number;
	readonly inner: T;
}
interface RspackThreadsafeResult<T> {
	readonly id: number;
	readonly inner: T;
}
const createDummyResult = (id: number): string => {
	const result: RspackThreadsafeResult<null> = {
		id,
		inner: null
	};
	return JSON.stringify(result);
};
class Rspack {
	#instance: ExternalObject<RspackInternal>;
	#plugins: RspackOptions["plugins"];
	hooks: {
		processAssets: tapable.AsyncSeriesHook<Assets>;
		done: tapable.AsyncSeriesHook<void>;
	};
	options: ResolvedRspackOptions;
	constructor(options: RspackOptions = {}) {
		this.options = config.resolveConfig(options);

		//@ts-ignored TODO: fix it later
		this.#instance = binding.newRspack(this.options, {
			doneCallback: this.#done.bind(this),
			processAssetsCallback: this.#procssAssets.bind(this)
		});
		this.hooks = {
			processAssets: new tapable.AsyncSeriesHook<Assets>(["assets"]),
			done: new tapable.AsyncSeriesHook<void>()
		};
		this.#plugins = options.plugins ?? [];
		for (const plugin of this.#plugins) {
			plugin.apply(this);
		}
	}
	async #done(err: Error, value: string) {
		if (err) {
			throw err;
		}
		const context: RspackThreadsafeContext<void> = JSON.parse(value);
		await this.hooks.done.promise();
		return createDummyResult(context.id);
	}
	async #procssAssets(err: Error, value: string) {
		if (err) {
			throw err;
		}
		const context: RspackThreadsafeContext<Assets> = JSON.parse(value);
		await this.hooks.processAssets.promise(context?.inner);
		return createDummyResult(context.id);
	}
	async build() {
		const stats = await binding.build(this.#instance);
		return stats;
	}

	async rebuild(changedFilesPath: string[]) {
		const stats = await binding.rebuild(this.#instance, changedFilesPath);
		return stats;
	}
	updateAsset(filename: string, asset: Config.Asset) {
		console.log("xx:", filename, asset);
		binding.updateAsset(this.#instance, {
			filename,
			asset
		});
	}
}

export { Rspack };
export default Rspack;
