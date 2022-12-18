import * as tapable from "tapable";
import { RawSource, Source } from "webpack-sources";
import { Resolver } from "enhanced-resolve";

import {
	JsCompilation,
	JsAssetInfo,
	JsCompatSource,
	JsAsset
} from "@rspack/binding";

import { RspackOptionsNormalized } from "./config";
import { createRawFromSource, createSourceFromRaw } from "./util/createSource";
import { ResolvedOutput } from "./config/output";
import { ChunkGroup } from "./chunk_group";
import { Compiler } from "./compiler";
import ResolverFactory from "./ResolverFactory";

const hashDigestLength = 8;
const EMPTY_ASSET_INFO = {};

export type AssetInfo = Partial<JsAssetInfo> & Record<string, any>;

export class Compilation {
	#inner: JsCompilation;

	hooks: {
		processAssets: tapable.AsyncSeriesHook<Record<string, Source>>;
	};
	options: RspackOptionsNormalized;
	outputOptions: ResolvedOutput;
	compiler: Compiler;
	resolverFactory: ResolverFactory;

	constructor(compiler: Compiler, inner: JsCompilation) {
		this.hooks = {
			processAssets: new tapable.AsyncSeriesHook<Record<string, Source>>([
				"assets"
			])
		};
		this.compiler = compiler;
		this.options = compiler.options;
		this.outputOptions = compiler.options.output;
		this.#inner = inner;
	}

	get hash() {
		return this.#inner.hash;
	}

	get fullHash() {
		return this.#inner.hash;
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
	 * Get a map of all entrypoints.
	 */
	get entrypoints(): Map<string, ChunkGroup> {
		return new Map(
			Object.entries(this.#inner.entrypoints).map(([n, e]) => [
				n,
				new ChunkGroup(e)
			])
		);
	}

	/**
	 * Update an existing asset. Trying to update an asset that doesn't exist will throw an error.
	 *
	 * See: [Compilation.updateAsset](https://webpack.js.org/api/compilation-object/#updateasset)
	 * Source: [updateAsset](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4320)
	 *
	 * FIXME: *AssetInfo* may be undefined in update fn for webpack impl, but still not implemented in rspack
	 *
	 * @param {string} file file name
	 * @param {Source | function(Source): Source} newSourceOrFunction new asset source or function converting old to new
	 * @param {JsAssetInfo | function(JsAssetInfo): JsAssetInfo} assetInfoUpdateOrFunction new asset info or function converting old to new
	 */
	updateAsset(
		filename: string,
		newSourceOrFunction: Source | ((source: Source) => Source),
		assetInfoUpdateOrFunction:
			| JsAssetInfo
			| ((assetInfo: JsAssetInfo) => JsAssetInfo)
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
	 * @param {JsAssetInfo} assetInfo extra asset information
	 * @returns {void}
	 */
	emitAsset(filename: string, source: Source, assetInfo?: AssetInfo) {
		const info = Object.assign(
			{
				minimized: false,
				development: false,
				hotModuleReplacement: false,
				related: {}
			},
			assetInfo
		);
		this.#inner.emitAsset(filename, createRawFromSource(source), info);
	}

	deleteAsset(filename: string) {
		this.#inner.deleteAsset(filename);
	}

	/**
	 * Get an array of Asset
	 *
	 * See: [Compilation.getAssets](https://webpack.js.org/api/compilation-object/#getassets)
	 * Source: [getAssets](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4448)
	 *
	 * @return {Readonly<JsAsset>[]}
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

	getAsset(name: string) {
		const asset = this.#inner.getAsset(name);
		if (!asset) {
			return;
		}
		return {
			...asset,
			source: createSourceFromRaw(asset.source)
		};
	}

	pushDiagnostic(
		severity: "error" | "warning",
		title: string,
		message: string
	) {
		this.#inner.pushDiagnostic(severity, title, message);
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

	/**
	 * Get the `Source` of an given asset filename.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getAssetSource(filename: string): Source | null {
		return createSourceFromRaw(this.#inner.getAssetSource(filename));
	}

	/**
	 * Get a list of asset filenames.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getAssetFilenames(): string[] {
		return this.#inner.getAssetFilenames();
	}

	/**
	 * Test if an asset exists.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__hasAsset(name: string): boolean {
		return this.#inner.hasAsset(name);
	}

	createStats() {
		return {};
	}

	seal() {}
	unseal() {}
}

export type { JsAssetInfo };
