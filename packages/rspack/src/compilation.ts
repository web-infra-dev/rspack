import * as tapable from "tapable";
import { RspackOptionsNormalized } from "./config";
import { RawSource, Source } from "webpack-sources";
import { EmitAssetCallback } from "./compiler";
import { createHash } from "./utils/createHash";
export type Asset = {
	source: Source;
	name: string;
	info: AssetInfo;
};
export type Assets = Record<string, Asset>;
const hashDigestLength = 8;
type CompilationAssets = Record<string, Source>;
type KnownAssetInfo = Object;
type AssetInfo = KnownAssetInfo & Record<string, any>;
const EMPTY_ASSET_INFO = Object.freeze({});
export class Compilation {
	#emitAssetCallback: EmitAssetCallback;
	hooks: {
		processAssets: tapable.AsyncSeriesHook<Record<string, Source>>;
	};
	fullHash: string;
	hash: string;
	options: RspackOptionsNormalized;
	assets: CompilationAssets;
	assetsInfo: Map<string, Map<string, Set<string>>>;
	constructor(options: RspackOptionsNormalized) {
		this.hooks = {
			processAssets: new tapable.AsyncSeriesHook<Record<string, Source>>([
				"assets"
			])
		};
		this.options = options;
		const hash = createHash(this.options.output.hashFunction);
		this.fullHash = hash.digest(options.output.hashDigest);
		this.hash = this.fullHash.slice(0, hashDigestLength);
		this.assets = {};
		this.assetsInfo = new Map();
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
			throw new Error("can't call emitAsset out of processAssets hook for now");
		}
		this.#emitAssetCallback({
			filename: filename,
			asset
		});
	}
	async processAssets(value: string, emitAsset: any) {
		this.#emitAssetCallback = emitAsset;
		let content: Record<string, number[]> = JSON.parse(value) ?? {};
		let assets = {};
		for (const [key, value] of Object.entries(content)) {
			let buffer = Buffer.from(value);
			// webpack-sources's type definition is wrong, it actually could accept Buffer type
			assets[key] = new RawSource(buffer as any);
		}
		await this.hooks.processAssets.promise(assets);
	}
	createStats() {
		return {};
	}
	getPath(filename: string, data: Record<string, any> = {}) {
		if (!data.hash) {
			data = {
				hash: this.hash,
				...data
			};
		}
		return this.getAssetPath(filename, data);
	}
	getAssetPath(filename, data) {
		return filename;
	}
	getAssets() {
		const array: Readonly<Asset>[] = [];
		for (const assetName of Object.keys(this.assets)) {
			array.push({
				name: assetName,
				source: this.assets[assetName],
				info: this.assetsInfo.get(assetName) || EMPTY_ASSET_INFO
			});
		}
		return array;
	}
	seal() {}
	unseal() {}
}
