export * from "./build";
import * as binding from "@rspack/binding";
import type { ExternalObject, RspackInternal } from "@rspack/binding";
import * as tapable from "tapable";
import * as config from "./config";
import { RspackOptions, Assets, Asset, User2Native } from "./config";
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
type EmitAssetCallback = (options: { filename: string; asset: Asset }) => void;
class RspackCompilation {
	#emitAssetCallback: EmitAssetCallback;
	hooks: {
		processAssets: tapable.AsyncSeriesHook<Assets>;
	};
	constructor() {
		this.hooks = {
			processAssets: new tapable.AsyncSeriesHook<Assets>(["assets"])
		};
	}
	/**
	 * unsafe to call out of processAssets
	 * @param filename
	 * @param asset
	 */
	updateAsset(filename: string, asset: Asset) {
		this.emitAsset(filename, asset);
	}
	/**
	 * unsafe to call out of processAssets
	 * @param filename
	 * @param asset
	 */
	emitAsset(filename: string, asset: Asset) {
		if (!this.#emitAssetCallback) {
			throw new Error("can't call emitAsset outof processAssets hook for now");
		}
		this.#emitAssetCallback({
			filename: filename,
			asset
		});
	}
	async procssAssets(err: Error, value: string, emitAsset: any) {
		this.#emitAssetCallback = emitAsset;
		if (err) {
			throw err;
		}
		const context: RspackThreadsafeContext<Assets> = JSON.parse(value);
		await this.hooks.processAssets.promise(context?.inner);
		return createDummyResult(context.id);
	}
}
class Rspack {
	#plugins: RspackOptions["plugins"];
	#instance: ExternalObject<RspackInternal>;
	compilation: RspackCompilation;
	hooks: {
		done: tapable.AsyncSeriesHook<void>;
		compilation: tapable.SyncHook<RspackCompilation>;
	};
	constructor(public options: RspackOptions) {
		const nativeConfig = User2Native(options);
		this.#instance = binding.newRspack(nativeConfig, {
			doneCallback: this.#done.bind(this),
			processAssetsCallback: this.#procssAssets.bind(this)
		});
		this.hooks = {
			done: new tapable.AsyncSeriesHook<void>(),
			compilation: new tapable.SyncHook<RspackCompilation>(["compilation"])
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
	async #procssAssets(err: Error, value: string, emitAsset: any) {
		return this.compilation.procssAssets(err, value, emitAsset);
	}
	#newCompilation() {
		const compilation = new RspackCompilation();
		this.compilation = compilation;
		this.hooks.compilation.call(compilation);
		return compilation;
	}
	async build() {
		const compilation = this.#newCompilation();
		const stats = await binding.build(this.#instance);
		return stats;
	}
	async rebuild(changeFiles: string[]) {
		const stats = await binding.rebuild(this.#instance, changeFiles);
		return stats;
	}
}
export { Rspack };
export default Rspack;
