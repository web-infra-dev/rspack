import * as tapable from "tapable";
import { RawSource, Source } from "webpack-sources";

import { JsCompilation, AssetInfo, JsCompatSource } from "@rspack/binding";

import { createHash } from "./utils/createHash";
import { RspackOptionsNormalized } from "./config";
import { createRawFromSource, createSourceFromRaw } from "./utils/createSource";

const hashDigestLength = 8;

export class Compilation {
	#inner: JsCompilation;

	hooks: {
		processAssets: tapable.AsyncSeriesHook<Record<string, Source>>;
	};
	fullHash: string;
	hash: string;
	options: RspackOptionsNormalized;

	constructor(options: RspackOptionsNormalized, inner: JsCompilation) {
		this.hooks = {
			processAssets: new tapable.AsyncSeriesHook<Record<string, Source>>([
				"assets"
			])
		};
		this.options = options;
		const hash = createHash(this.options.output.hashFunction);
		this.fullHash = hash.digest(options.output.hashDigest);
		this.hash = this.fullHash.slice(0, hashDigestLength);
		this.#inner = inner;
	}

	/**
	 * Get a map of all assets.
	 *
	 * Source: [assets](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L1008-L1009)
	 */
	get assets(): Record<string, Source> {
		const iterator = Object.entries(this.#inner.assets).map(
			([filename, source]) => [filename, createSourceFromRaw(source)]
		);

		return Object.fromEntries(iterator);
	}

	/**
	 * Update an existing asset. Trying to update an asset that doesn't exist will throw an error.
	 *
	 * See: [Compilation.updateAsset](https://webpack.js.org/api/compilation-object/#updateasset)
	 * Source: [updateAsset](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4320)
	 *
	 * @param {string} file file name
	 * @param {Source | function(Source): Source} newSourceOrFunction new asset source or function converting old to new
	 * @param {AssetInfo | function(AssetInfo): AssetInfo} assetInfoUpdateOrFunction new asset info or function converting old to new, FIXME: *AssetInfo* may be undefined in update fn for webpack impl, but still not implemented in rspack
	 */
	updateAsset(
		filename: string,
		newSourceOrFunction: Source | ((source: Source) => Source),
		assetInfoUpdateOrFunction: AssetInfo | ((assetInfo: AssetInfo) => AssetInfo)
	) {
		let compatNewSourceOrFunction:
			| JsCompatSource
			| ((source: JsCompatSource) => JsCompatSource);

		if (typeof newSourceOrFunction === "function") {
			compatNewSourceOrFunction = function newSourceFunction(
				source: JsCompatSource
			) {
				return createRawFromSource(
					newSourceOrFunction(createSourceFromRaw(source))
				);
			};
		} else {
			compatNewSourceOrFunction = createRawFromSource(newSourceOrFunction);
		}

		this.#inner.updateAsset(
			filename,
			compatNewSourceOrFunction,
			assetInfoUpdateOrFunction
		);
	}

	/**
	 * Emit an not existing asset. Trying to emit an asset that already exists will throw an error.
	 *
	 * See: [Compilation.emitAsset](https://webpack.js.org/api/compilation-object/#emitasset)
	 * Source: [emitAsset](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4239)
	 *
	 * @param {string} file file name
	 * @param {Source} source asset source
	 * @param {AssetInfo} assetInfo extra asset information
	 * @returns {void}
	 */
	emitAsset(
		filename: string,
		source: Source,
		assetInfo: AssetInfo = {
			minimized: false,
			development: false,
			related: {}
		}
	) {
		this.#inner.emitAsset(filename, createRawFromSource(source), assetInfo);
	}

	/**
	 * Get an array of Asset
	 *
	 * See: [Compilation.getAssets](https://webpack.js.org/api/compilation-object/#getassets)
	 * Source: [getAssets](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4448)
	 *
	 * @return {Readonly<Asset>[]}
	 */
	getAssets() {
		const assets = this.#inner.getAssets();

		return assets.map(asset => {
			const source = createSourceFromRaw(asset.source);
			return {
				...asset,
				source
			};
		});
	}

	// TODO: full alignment
	getPath(filename: string, data: Record<string, any> = {}) {
		if (!data.hash) {
			data = {
				hash: this.hash,
				...data
			};
		}
		return this.getAssetPath(filename, data);
	}

	// TODO: full alignment
	getAssetPath(filename, data) {
		return filename;
	}

	createStats() {
		return {};
	}

	seal() {}
	unseal() {}
}
