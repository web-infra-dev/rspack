import * as binding from "@rspack/binding";
import { resolveWatchOption } from "./config/watch";
import type { Watch, ResolvedWatch } from "./config/watch";

import type { ExternalObject, RspackInternal } from "@rspack/binding";
import * as tapable from "tapable";
import { SyncHook } from "tapable";
import {
	RspackOptions,
	ResolvedRspackOptions,
	Asset,
	resolveOptions
} from "./config";

import { Stats } from "./stats";
import { Compilation } from "./compilation";
export interface RspackThreadsafeContext<T> {
	readonly id: number;
	readonly inner: T;
}
interface RspackThreadsafeResult<T> {
	readonly id: number;
	readonly inner: T;
}
export const createDummyResult = (id: number): string => {
	const result: RspackThreadsafeResult<null> = {
		id,
		inner: null
	};
	return JSON.stringify(result);
};
export type EmitAssetCallback = (options: {
	filename: string;
	asset: Asset;
}) => void;
class EntryPlugin {
	apply() {}
}
class HotModuleReplacementPlugin {
	apply() {}
}
class Compiler {
	webpack: any;
	#plugins: RspackOptions["plugins"];
	#instance: ExternalObject<RspackInternal>;
	compilation: Compilation;
	hooks: {
		done: tapable.AsyncSeriesHook<Stats>;
		compilation: tapable.SyncHook<Compilation>;
		invalid: tapable.SyncHook<[string | null, number]>;
		compile: tapable.SyncHook<[any]>;
		initialize: tapable.SyncHook<[]>;
	};
	options: ResolvedRspackOptions;
	getInfrastructureLogger(name: string) {
		return {
			info: msg => console.info(msg)
		};
	}
	constructor(context: string, options: ResolvedRspackOptions) {
		this.options = options;
		// to workaround some plugin access webpack, we may change dev-server to avoid this hack in the future
		this.webpack = {
			EntryPlugin, // modernjs/server use this to inject dev-client
			HotModuleReplacementPlugin // modernjs/server will auto inject this this plugin not set
		};
		// @ts-ignored
		this.#instance = binding.newRspack(this.options, {
			doneCallback: this.#done.bind(this),
			processAssetsCallback: this.#processAssets.bind(this)
		});
		this.hooks = {
			done: new tapable.AsyncSeriesHook<Stats>(["stats"]),
			compilation: new tapable.SyncHook<Compilation>(["compilation"]),
			invalid: new SyncHook(["filename", "changeTime"]),
			compile: new SyncHook(["params"]),
			initialize: new SyncHook([])
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
		// @todo context.inner is empty, since we didn't pass to binding
		const stats = new Stats(context.inner);
		await this.hooks.done.promise(stats);
		return createDummyResult(context.id);
	}
	async #processAssets(err: Error, value: string, emitAsset: any) {
		return this.compilation.processAssets(err, value, emitAsset);
	}
	#newCompilation() {
		const compilation = new Compilation();
		this.compilation = compilation;
		this.hooks.compilation.call(compilation);
		return compilation;
	}
	async build() {
		const compilation = this.#newCompilation();
		const stats = await binding.build(this.#instance);
		return stats;
	}
	async rebuild() {
		const stats = await binding.rebuild(this.#instance);
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
			const diffStats = await this.rebuild();
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

export { Compiler };
