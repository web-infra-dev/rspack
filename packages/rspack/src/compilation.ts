import * as tapable from "tapable";
import { Asset } from "./config";
import { RawSource, Source } from "webpack-sources";
import {
	EmitAssetCallback,
	RspackThreadsafeContext,
	createDummyResult
} from "./compiler";

export class Compilation {
	#emitAssetCallback: EmitAssetCallback;
	hooks: {
		processAssets: tapable.AsyncSeriesHook<Record<string, Source>>;
	};
	constructor() {
		this.hooks = {
			processAssets: new tapable.AsyncSeriesHook<Record<string, Source>>([
				"assets"
			])
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
	async processAssets(err: Error, value: string, emitAsset: any) {
		this.#emitAssetCallback = emitAsset;
		if (err) {
			throw err;
		}
		const context: RspackThreadsafeContext<Record<string, number[]>> =
			JSON.parse(value);
		let content: Record<string, number[]> = context.inner ?? {};
		let assets = {};
		for (const [key, value] of Object.entries(content)) {
			let buffer = Buffer.from(value);
			// webpack-sources's type definition is wrong, it actually could accept Buffer type
			assets[key] = new RawSource(buffer as any);
		}
		await this.hooks.processAssets.promise(assets);
		return createDummyResult(context.id);
	}
}
