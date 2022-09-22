export * from "./build";
import * as binding from "@rspack/binding";
import { resolveWatchOption } from "./config/watch";
import type { Watch, ResolvedWatch } from "./config/watch";

import type { ExternalObject, RspackInternal } from "@rspack/binding";
import * as tapable from "tapable";
import {
	RspackOptions,
	ResolvedRspackOptions,
	Asset,
	resolveOptions
} from "./config";

import { RawSource, Source } from "webpack-sources";
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
class Rspack {
	#plugins: RspackOptions["plugins"];
	#instance: ExternalObject<RspackInternal>;
	compilation: RspackCompilation;
	hooks: {
		done: tapable.AsyncSeriesHook<void>;
		compilation: tapable.SyncHook<RspackCompilation>;
	};
	options: ResolvedRspackOptions;
	constructor(private userOptions: RspackOptions) {
		this.options = resolveOptions(userOptions);
		// @ts-ignored
		this.#instance = binding.newRspack(this.options, {
			doneCallback: this.#done.bind(this),
			processAssetsCallback: this.#processAssets.bind(this)
		});
		this.hooks = {
			done: new tapable.AsyncSeriesHook<void>(),
			compilation: new tapable.SyncHook<RspackCompilation>(["compilation"])
		};
		this.#plugins = userOptions.plugins ?? [];
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
	async #processAssets(err: Error, value: string, emitAsset: any) {
		return this.compilation.processAssets(err, value, emitAsset);
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

	async watch(watchOptions?: Watch): Promise<Watching> {
		const options = resolveWatchOption(watchOptions);

		const watcher = (await import("chokidar")).default.watch(
			this.options.context,
			{
				ignoreInitial: true,
				...options
			}
		);
		let stats = await this.build();

		watcher.on("change", async () => {
			// TODO: only build because we lack the snapshot info of file.
			// TODO: it means there a lot of things to do....
			const begin = Date.now();
			console.log("hit change and start to build");
			const compiler = new Rspack(this.userOptions);
			stats = await compiler.build();
			console.log("build success, time cost", Date.now() - begin);
		});

		return {
			async close() {
				await watcher.close();
			}
		};
	}
}

export interface Watching {
	close(): Promise<void>;
}

export { Rspack };
